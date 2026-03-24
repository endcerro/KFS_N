// memory/allocator.rs - GlobalAlloc implementation for the kernel heap
//
// Bridges Rust's `alloc` crate (Box, Vec, String, etc.) to our
// kmalloc/kfree free-list allocator.
//
// Alignment handling:
//   kmalloc() guarantees 8-byte alignment (ALLOC_ALIGN).  When the
//   Layout requests a higher alignment, we over-allocate, find an
//   aligned position within the buffer, and stash the *original*
//   kmalloc pointer just before the aligned position so kfree() can
//   recover it.
//
//   For align <= ALLOC_ALIGN we fast-path straight to kmalloc/kfree
//   with zero overhead.
//
//   ┌─ kmalloc'd block ──────────────────────────────────┐
//   │  ... padding ...  │ original_ptr │  aligned region  │
//   └──────────────────────────────────┘──────────────────┘
//                                      ▲
//                                      └─ pointer returned to caller
//
//   `original_ptr` is a usize stored at (aligned - size_of::<usize>()).
//   This is always inside the padding because align > ALLOC_ALIGN >= 8
//   guarantees at least `align` bytes of padding room.
//
// Forward-compatibility for user space:
//   This module only handles kernel-side allocation.  When user-space
//   processes arrive, each process will have its own heap region and
//   allocator instance.  The GlobalAlloc registered here stays as the
//   kernel allocator; user processes will use a different mechanism
//   (e.g. a syscall-backed allocator mapped into their address space).
//   The KernelAllocator struct is intentionally stateless so swapping
//   in a process-aware allocator later is straightforward.

use super::heap;
use core::alloc::{GlobalAlloc, Layout};

// ---------------------------------------------------------------------------
// The minimum alignment that kmalloc guarantees.  Must match heap.rs.
// ---------------------------------------------------------------------------
const KMALLOC_ALIGN: usize = 8;

// ---------------------------------------------------------------------------
// KernelAllocator - zero-sized struct registered as #[global_allocator]
// ---------------------------------------------------------------------------
//
// Stateless: all state lives in heap.rs statics.  This keeps the door
// open for a future per-address-space allocator without touching this
// trait impl.
pub struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        if size == 0 {
            // GlobalAlloc contract: zero-sized allocations may return
            // a non-null, dangling, well-aligned pointer.  We use the
            // alignment value itself as a sentinel (never dereferenced).
            return align as *mut u8;
        }

        if align <= KMALLOC_ALIGN {
            // Fast path: kmalloc already provides this alignment.
            return heap::kmalloc(size);
        }

        // Slow path: over-allocate so we can find an aligned position
        // and stash the original pointer for dealloc().
        //
        // We need `size` usable bytes at an `align`-aligned address,
        // plus room to store the original pointer (1 usize) just before
        // that address.  Worst-case waste is `align - 1` bytes of
        // padding plus the usize.
        let overhead = align - 1 + core::mem::size_of::<usize>();
        let total = size + overhead;

        let raw = heap::kmalloc(total);
        if raw.is_null() {
            return core::ptr::null_mut();
        }

        // Find the first `align`-aligned address at or after
        // (raw + sizeof(usize)).  The +sizeof(usize) reserves room
        // for the stashed pointer.
        let raw_addr = raw as usize;
        let min_start = raw_addr + core::mem::size_of::<usize>();
        let aligned_addr = (min_start + align - 1) & !(align - 1);

        // Stash the original kmalloc pointer right before the aligned
        // address so dealloc() can recover it.
        let stash = (aligned_addr - core::mem::size_of::<usize>()) as *mut usize;
        stash.write(raw_addr);

        aligned_addr as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() == 0 {
            // Matches the zero-size sentinel from alloc() - nothing to free.
            return;
        }

        if layout.align() <= KMALLOC_ALIGN {
            // Fast path: ptr came directly from kmalloc.
            heap::kfree(ptr);
            return;
        }

        // Slow path: recover the original kmalloc pointer from the
        // stash slot just before `ptr`.
        let stash = (ptr as usize - core::mem::size_of::<usize>()) as *const usize;
        let original = stash.read() as *mut u8;
        heap::kfree(original);
    }
}

// ---------------------------------------------------------------------------
// Alloc error handler
// ---------------------------------------------------------------------------
//
// Called by the alloc crate when an allocation fails (e.g. Vec::push
// when the heap is exhausted).  In a kernel, this is a fatal error.
#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!(
        "kernel heap allocation failed: size={}, align={}",
        layout.size(),
        layout.align()
    );
}

// ---------------------------------------------------------------------------
// Self-test: exercises Box, Vec, and String via the global allocator
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc_test")]
pub fn test_global_alloc() {
    extern crate alloc;
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;

    println!("\n=== GlobalAlloc Self-Test ===\n");

    // Test 1: Box<u32>
    print!("[Alloc test 1] Box<u32> ... ");
    {
        let b = Box::new(42u32);
        assert_eq!(*b, 42);
        // b is dropped here, exercising dealloc
    }
    println!("OK");

    // Test 2: Vec with push and indexing
    print!("[Alloc test 2] Vec<u32> push/grow ... ");
    {
        let mut v: Vec<u32> = Vec::new();
        for i in 0..100 {
            v.push(i);
        }
        assert_eq!(v.len(), 100);
        assert_eq!(v[0], 0);
        assert_eq!(v[99], 99);
        // Vec reallocates several times during this loop, testing
        // alloc + dealloc + realloc paths.
    }
    println!("OK");

    // Test 3: String
    print!("[Alloc test 3] String ... ");
    {
        let mut s = String::from("Hello");
        s.push_str(", kernel world!");
        assert_eq!(s.as_str(), "Hello, kernel world!");
    }
    println!("OK");

    // Test 4: vec! macro (uses alloc internally)
    print!("[Alloc test 4] vec! macro ... ");
    {
        let v = vec![1u8, 2, 3, 4, 5];
        assert_eq!(v.len(), 5);
        assert_eq!(v.iter().sum::<u8>(), 15);
    }
    println!("OK");

    // Test 5: Large allocation (exercises heap growth)
    print!("[Alloc test 5] Large Vec (4096 elements) ... ");
    {
        let v: Vec<u32> = (0..4096).collect();
        assert_eq!(v.len(), 4096);
        assert_eq!(v[4095], 4095);
    }
    println!("OK");

    // Test 6: Over-aligned allocation (e.g. SIMD-like)
    print!("[Alloc test 6] Over-aligned Box ... ");
    {
        // Force a 64-byte aligned allocation via Layout
        let layout = Layout::from_size_align(128, 64).unwrap();
        let ptr = unsafe { alloc::alloc::alloc(layout) };
        assert!(!ptr.is_null(), "Over-aligned alloc returned null");
        assert_eq!(
            ptr as usize % 64,
            0,
            "Over-aligned pointer not 64-byte aligned"
        );
        unsafe {
            // Write and readback
            core::ptr::write_volatile(ptr as *mut u32, 0xCAFE_BABE);
            let val = core::ptr::read_volatile(ptr as *const u32);
            assert_eq!(val, 0xCAFE_BABE);
            alloc::alloc::dealloc(ptr, layout);
        }
    }
    println!("OK");

    println!("\n=== GlobalAlloc Self-Test PASSED ===\n");
}
