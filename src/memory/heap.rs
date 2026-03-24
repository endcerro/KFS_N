// memory/heap.rs - Kernel heap: region management + linked-list free-list allocator
//
// Memory layout of an allocated block:
//
//   ┌──────────────────────┐  ◄─ block start (what the free list tracks)
//   │  BlockHeader         │     size: usable bytes after header
//   │    size: usize       │     is_free: false for allocated blocks
//   │    is_free: bool     │     next: unused while allocated
//   │    next: *mut Header │
//   ├──────────────────────┤  ◄─ pointer returned by kmalloc()
//   │                      │
//   │  usable memory       │     `size` bytes
//   │                      │
//   └──────────────────────┘
//
// Free blocks are identical but is_free=true and `next` points to the
// next free block (or null).  The free list is sorted by address so
// that adjacent-block merging in kfree() is straightforward.
//
// When no free block is large enough, the allocator automatically grows
// the heap by mapping more pages, creates a new free block in the fresh
// region, and merges it with the last block if adjacent.
//
// Alignment: all allocations are aligned to ALLOC_ALIGN (8 bytes on
// 32-bit).  The header size is rounded up to ALLOC_ALIGN so the
// usable region is always aligned.

use super::define::{KERNEL_HEAP_END, KERNEL_HEAP_INITIAL_SIZE, KERNEL_HEAP_START, PAGE_SIZE};
use super::pageflags::PageFlags;
use super::vmm::{self, MapError, VirtAddr};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// Minimum alignment for all allocations (and the header itself).
// 8 bytes keeps u64 and pointer-pair structs happy on i386.
const ALLOC_ALIGN: usize = 8;

// Minimum usable block size.  Blocks smaller than this aren't worth
// splitting - they'd just become unfindable fragments.
const MIN_BLOCK_SIZE: usize = ALLOC_ALIGN;

// How much to grow the heap when we run out of space (minimum).
// One page (4 KB) at a time keeps waste low while avoiding excessive
// map_range calls for tight loops of small allocations.
const GROW_INCREMENT: usize = PAGE_SIZE;

// ---------------------------------------------------------------------------
// Block header
// ---------------------------------------------------------------------------

// Header placed immediately before every block (free or allocated).
// Kept small so overhead per allocation is minimal.
//
// SAFETY: this struct is written into raw heap memory via pointer casts.
// It must have a stable layout - hence #[repr(C)].
#[repr(C)]
struct BlockHeader {
    // Size of the usable region *after* this header, in bytes.
    size: usize,
    // True if this block is on the free list.
    is_free: bool,
    // Next free block (only meaningful when is_free == true).
    next: *mut BlockHeader,
}

// Header size, rounded up to ALLOC_ALIGN so the usable region that
// follows is always aligned.
const HEADER_SIZE: usize =
    (core::mem::size_of::<BlockHeader>() + ALLOC_ALIGN - 1) & !(ALLOC_ALIGN - 1);

// ---------------------------------------------------------------------------
// Global state
// ---------------------------------------------------------------------------

// Head of the free list (sorted by address, lowest first).
static mut FREE_LIST: *mut BlockHeader = core::ptr::null_mut();

// Current end of the mapped heap region.  Everything in
// [KERNEL_HEAP_START .. HEAP_MAPPED_END) is backed by physical frames.
static mut HEAP_MAPPED_END: usize = KERNEL_HEAP_START;

// Simple statistics - not required for correctness but useful for
// debugging and the print_stats() diagnostic.
static mut STATS: HeapStats = HeapStats {
    total_allocs: 0,
    total_frees: 0,
    current_used_bytes: 0,
};

struct HeapStats {
    total_allocs: usize,
    total_frees: usize,
    // Sum of usable sizes of currently-allocated blocks.
    current_used_bytes: usize,
}

// ---------------------------------------------------------------------------
// Initialisation
// ---------------------------------------------------------------------------

// Initialise the kernel heap: map the initial pages and set up the free
// list with a single large free block spanning the entire region.
//
// Must be called after vmm::init() and the physical frame allocator.
pub fn init() {
    let initial = KERNEL_HEAP_INITIAL_SIZE;
    assert!(
        initial <= KERNEL_HEAP_END - KERNEL_HEAP_START,
        "Initial heap size exceeds heap region"
    );
    assert!(
        initial % PAGE_SIZE == 0,
        "Initial heap size must be page-aligned"
    );
    assert!(
        initial > HEADER_SIZE,
        "Initial heap too small for even one block"
    );
    #[cfg(feature = "verbose")]
    println!(
        "Heap: mapping initial region {:#x}..{:#x} ({} KB)",
        KERNEL_HEAP_START,
        KERNEL_HEAP_START + initial,
        initial / 1024
    );

    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;
    let _pages: usize = vmm::map_range(VirtAddr::new(KERNEL_HEAP_START as u32), initial, flags)
        .expect("Heap: failed to map initial region");

    unsafe {
        HEAP_MAPPED_END = KERNEL_HEAP_START + initial;

        // Create a single free block covering the entire mapped region.
        let first_block = KERNEL_HEAP_START as *mut BlockHeader;
        (*first_block).size = initial - HEADER_SIZE;
        (*first_block).is_free = true;
        (*first_block).next = core::ptr::null_mut();
        FREE_LIST = first_block;
    }

    #[cfg(feature = "verbose")]
    println!(
        "Heap: {} pages mapped, allocator ready ({} KB usable)",
        pages,
        (initial - HEADER_SIZE) / 1024
    );
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

// Allocate `size` bytes from the kernel heap.
//
// Returns a pointer to the usable region (after the header), or null
// if the allocation cannot be satisfied even after growing the heap.
//
// The returned pointer is aligned to ALLOC_ALIGN (8 bytes).
pub fn kmalloc(size: usize) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }

    // Round up to alignment so all blocks stay aligned
    let aligned_size = align_up(size, ALLOC_ALIGN);

    unsafe {
        // First-fit search through the free list
        if let Some(block) = find_free_block(aligned_size) {
            return allocate_block(block, aligned_size);
        }

        // No block large enough - try to grow the heap
        if try_grow_for(aligned_size).is_ok() {
            // Retry after growing
            if let Some(block) = find_free_block(aligned_size) {
                return allocate_block(block, aligned_size);
            }
        }

        // Truly out of memory
        println!("kmalloc: out of memory (requested {} bytes)", size);
        core::ptr::null_mut()
    }
}

// Free a previously allocated block.
//
// `ptr` must be a pointer returned by kmalloc().  Passing null is a
// safe no-op.  Double-free is detected and logged.
pub fn kfree(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        // The header sits immediately before the usable region
        let header = (ptr as usize - HEADER_SIZE) as *mut BlockHeader;

        // Sanity checks
        debug_assert!(
            (header as usize) >= KERNEL_HEAP_START && (header as usize) < HEAP_MAPPED_END,
            "kfree: pointer {:#x} outside heap region",
            ptr as usize
        );

        if (*header).is_free {
            println!("kfree: double free detected at {:#x}", ptr as usize);
            return;
        }

        // Mark as free and update stats
        (*header).is_free = true;
        STATS.total_frees += 1;
        STATS.current_used_bytes -= (*header).size;

        // Re-insert into the free list (sorted by address) and merge
        // with adjacent free blocks to reduce fragmentation.
        insert_free_block(header);
    }
}

// Return the usable size of an allocation (the size that was rounded
// up to alignment, which is >= the size originally requested).
//
// `ptr` must be a pointer returned by kmalloc().
pub fn ksize(ptr: *mut u8) -> usize {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let header = (ptr as usize - HEADER_SIZE) as *const BlockHeader;
        (*header).size
    }
}

// Grow the mapped heap region by `size` bytes (must be page-aligned).
//
// The new memory is added to the free list and merged with the tail
// block if adjacent.  Returns the start address of the newly mapped
// region, or an error if the heap would exceed KERNEL_HEAP_END or
// the frame allocator is exhausted.
//
// This is useful for pre-growing the heap before a burst of
// allocations.  Normal callers don't need this - kmalloc() grows
// automatically via try_grow_for().
pub fn grow_heap(size: usize) -> Result<usize, MapError> {
    assert!(
        size % PAGE_SIZE == 0,
        "grow_heap: size must be page-aligned"
    );

    unsafe { grow_mapped_region(size) }
}

// ---------------------------------------------------------------------------
// Internal: heap growth (single implementation)
// ---------------------------------------------------------------------------

// The single implementation backing both the public grow_heap() and the
// internal try_grow_for().  Maps `size` bytes at HEAP_MAPPED_END,
// creates a free block in the new region, and merges it into the list.
//
// `size` must be page-aligned.  Caller is responsible for the assert.
//
// Returns the start address of the newly mapped region.
unsafe fn grow_mapped_region(size: usize) -> Result<usize, MapError> {
    let old_end = HEAP_MAPPED_END;
    let new_end = old_end + size;

    if new_end > KERNEL_HEAP_END {
        println!(
            "Heap: cannot grow past {:#x} (requested {:#x})",
            KERNEL_HEAP_END, new_end
        );
        return Err(MapError::InvalidAddress);
    }

    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;
    vmm::map_range(VirtAddr::new(old_end as u32), size, flags)?;
    HEAP_MAPPED_END = new_end;

    // Create a free block covering the new region
    let new_block = old_end as *mut BlockHeader;
    (*new_block).size = size - HEADER_SIZE;
    (*new_block).is_free = true;
    (*new_block).next = core::ptr::null_mut();

    // Insert into free list - will auto-merge with the previous tail
    // block if the old last free block ended exactly at old_end.
    insert_free_block(new_block);

    #[cfg(feature = "verbose")]
    println!(
        "Heap: grew by {} KB, mapped end now {:#x}",
        size / 1024,
        new_end
    );

    Ok(old_end)
}

// Called automatically by kmalloc() when no free block is large enough.
// Computes how much to grow (at least GROW_INCREMENT, or enough for the
// request) and delegates to grow_mapped_region().
unsafe fn try_grow_for(needed: usize) -> Result<(), MapError> {
    // We need HEADER_SIZE overhead for the new free block, plus the
    // requested amount.  Round up to whole pages.
    let total_needed = needed + HEADER_SIZE;
    let grow_size = align_up(
        if total_needed > GROW_INCREMENT {
            total_needed
        } else {
            GROW_INCREMENT
        },
        PAGE_SIZE,
    );

    grow_mapped_region(grow_size)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Internal: allocation
// ---------------------------------------------------------------------------

// Walk the free list and return the first block with size >= `needed`.
unsafe fn find_free_block(needed: usize) -> Option<*mut BlockHeader> {
    let mut current = FREE_LIST;
    while !current.is_null() {
        if (*current).is_free && (*current).size >= needed {
            return Some(current);
        }
        current = (*current).next;
    }
    None
}

// Mark `block` as allocated.  If the block is significantly larger
// than `needed`, split it so the remainder stays on the free list.
//
// Returns the usable-region pointer (header + HEADER_SIZE).
unsafe fn allocate_block(block: *mut BlockHeader, needed: usize) -> *mut u8 {
    // Try to split: only worth it if the remainder can hold a header
    // plus at least MIN_BLOCK_SIZE usable bytes.
    let remaining = (*block).size - needed;
    if remaining >= HEADER_SIZE + MIN_BLOCK_SIZE {
        // Create a new free block after the allocated region
        let new_block = (block as usize + HEADER_SIZE + needed) as *mut BlockHeader;
        (*new_block).size = remaining - HEADER_SIZE;
        (*new_block).is_free = true;
        (*new_block).next = (*block).next;

        (*block).size = needed;
        (*block).next = new_block;
    }

    // Remove from free list
    (*block).is_free = false;
    remove_from_free_list(block);

    STATS.total_allocs += 1;
    STATS.current_used_bytes += (*block).size;

    // Return pointer to usable region (right after the header)
    (block as usize + HEADER_SIZE) as *mut u8
}

// Remove a block from the free list.
unsafe fn remove_from_free_list(block: *mut BlockHeader) {
    if FREE_LIST == block {
        FREE_LIST = (*block).next;
        return;
    }

    let mut prev = FREE_LIST;
    while !prev.is_null() {
        if (*prev).next == block {
            (*prev).next = (*block).next;
            return;
        }
        prev = (*prev).next;
    }
}

// ---------------------------------------------------------------------------
// Internal: free + merge
// ---------------------------------------------------------------------------

// Insert a freed block back into the address-sorted free list, then
// merge with the previous and/or next block if they are adjacent.
//
// Merging is the key to avoiding fragmentation: if three 64-byte
// blocks are freed in sequence, they coalesce into a single ~192-byte
// block rather than staying as three separate entries.
unsafe fn insert_free_block(block: *mut BlockHeader) {
    let addr = block as usize;

    // Find insertion point: the block should go between `prev` and `next`
    // where prev.addr < block.addr < next.addr.
    let mut prev: *mut BlockHeader = core::ptr::null_mut();
    let mut current = FREE_LIST;

    while !current.is_null() && (current as usize) < addr {
        prev = current;
        current = (*current).next;
    }

    // Link block into the list
    (*block).next = current;
    if prev.is_null() {
        FREE_LIST = block;
    } else {
        (*prev).next = block;
    }

    // Merge with next block if adjacent in memory
    //   block_end = block_addr + HEADER_SIZE + block.size
    //   If block_end == current_addr, they're physically contiguous.
    if !current.is_null() {
        let block_end = addr + HEADER_SIZE + (*block).size;
        if block_end == current as usize {
            (*block).size += HEADER_SIZE + (*current).size;
            (*block).next = (*current).next;
        }
    }

    // Merge with previous block if adjacent in memory
    if !prev.is_null() {
        let prev_end = prev as usize + HEADER_SIZE + (*prev).size;
        if prev_end == addr {
            (*prev).size += HEADER_SIZE + (*block).size;
            (*prev).next = (*block).next;
        }
    }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

// Round `val` up to the next multiple of `align` (must be power of 2).
#[inline]
const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

// Returns the start of the usable heap region.
pub fn heap_start() -> usize {
    KERNEL_HEAP_START
}

// Returns the current end of the *mapped* heap region.
pub fn heap_mapped_end() -> usize {
    unsafe { HEAP_MAPPED_END }
}

// Returns the maximum possible heap end address.
pub fn heap_max() -> usize {
    KERNEL_HEAP_END
}

// Returns how many bytes are currently mapped for the heap.
pub fn heap_mapped_size() -> usize {
    unsafe { HEAP_MAPPED_END - KERNEL_HEAP_START }
}

// Print heap allocator statistics.
pub fn print_stats() {
    unsafe {
        let mapped = heap_mapped_size();
        let max = KERNEL_HEAP_END - KERNEL_HEAP_START;

        println!("Kernel Heap:");
        println!(
            "  Region:      {:#x}..{:#x} ({} KB max)",
            KERNEL_HEAP_START,
            KERNEL_HEAP_END,
            max / 1024
        );
        println!(
            "  Mapped:      {:#x}..{:#x} ({} KB)",
            KERNEL_HEAP_START,
            HEAP_MAPPED_END,
            mapped / 1024
        );
        println!("  Allocs:      {}", STATS.total_allocs);
        println!("  Frees:       {}", STATS.total_frees);
        println!("  In use:      {} bytes", STATS.current_used_bytes);

        // Walk free list to report total free space
        let mut free_bytes: usize = 0;
        let mut free_blocks: usize = 0;
        let mut current = FREE_LIST;
        while !current.is_null() {
            free_bytes += (*current).size;
            free_blocks += 1;
            current = (*current).next;
        }
        println!(
            "  Free:        {} bytes in {} block(s)",
            free_bytes, free_blocks
        );
    }
}

// ---------------------------------------------------------------------------
// Self-test suite
// ---------------------------------------------------------------------------

// Run after init() to verify the heap allocator works.
pub fn test_heap() {
    println!("\n=== Heap Allocator Self-Test ===\n");

    test_basic_alloc_free();
    test_multiple_allocs();
    test_merge_on_free();
    test_ksize();
    test_reuse_after_free();
    test_alignment();
    test_zero_alloc();

    println!("\n=== Heap Allocator Self-Test PASSED ===\n");
}

fn test_basic_alloc_free() {
    print!("[Heap test 1] Basic alloc/free ... ");

    let ptr = kmalloc(64);
    assert!(!ptr.is_null(), "kmalloc(64) returned null");
    assert!(
        ptr as usize >= KERNEL_HEAP_START && (ptr as usize) < unsafe { HEAP_MAPPED_END },
        "Pointer outside heap region"
    );

    // Write and read back through the allocated memory
    unsafe {
        core::ptr::write_volatile(ptr as *mut u32, 0xDEAD_BEEF);
        let val = core::ptr::read_volatile(ptr as *const u32);
        assert_eq!(val, 0xDEAD_BEEF, "Write/read mismatch");
    }

    kfree(ptr);
    println!("OK");
}

fn test_multiple_allocs() {
    print!("[Heap test 2] Multiple allocations ... ");

    let a = kmalloc(32);
    let b = kmalloc(64);
    let c = kmalloc(128);
    assert!(
        !a.is_null() && !b.is_null() && !c.is_null(),
        "Allocations failed"
    );

    // All pointers must be distinct
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);

    // Write unique markers and verify no cross-contamination
    unsafe {
        *(a as *mut u32) = 0xAAAA_AAAA;
        *(b as *mut u32) = 0xBBBB_BBBB;
        *(c as *mut u32) = 0xCCCC_CCCC;

        assert_eq!(*(a as *const u32), 0xAAAA_AAAA);
        assert_eq!(*(b as *const u32), 0xBBBB_BBBB);
        assert_eq!(*(c as *const u32), 0xCCCC_CCCC);
    }

    kfree(a);
    kfree(b);
    kfree(c);
    println!("OK");
}

fn test_merge_on_free() {
    print!("[Heap test 3] Merge on free ... ");

    // Allocate three blocks, then free them in a non-sequential order.
    // After all are freed the free list should merge them into one
    // contiguous block, allowing a larger allocation to succeed.
    let a = kmalloc(64);
    let b = kmalloc(64);
    let c = kmalloc(64);
    assert!(!a.is_null() && !b.is_null() && !c.is_null());

    // Free middle first, then sides - exercises both forward and
    // backward merge paths in insert_free_block().
    kfree(b);
    kfree(a);
    kfree(c);

    // Three 64-byte blocks + their headers should coalesce into one
    // chunk big enough for a 200-byte allocation.
    let big = kmalloc(200);
    assert!(
        !big.is_null(),
        "Merge failed - can't allocate 200 bytes after freeing 3x64"
    );
    kfree(big);

    println!("OK");
}

fn test_ksize() {
    print!("[Heap test 4] ksize ... ");

    let ptr = kmalloc(100);
    assert!(!ptr.is_null());

    let sz = ksize(ptr);
    // ksize returns the aligned size, which is >= requested
    assert!(sz >= 100, "ksize returned {} for a 100-byte alloc", sz);

    kfree(ptr);

    assert_eq!(ksize(core::ptr::null_mut()), 0, "ksize(null) should be 0");

    println!("OK");
}

fn test_reuse_after_free() {
    print!("[Heap test 5] Reuse after free ... ");

    let a = kmalloc(48);
    assert!(!a.is_null());
    let a_addr = a as usize;
    kfree(a);

    // A new allocation of the same size should reuse the same block
    // (first-fit will find it immediately at the head of the free list).
    let b = kmalloc(48);
    assert!(!b.is_null());
    assert_eq!(
        b as usize, a_addr,
        "Expected freed block to be reused (got {:#x}, expected {:#x})",
        b as usize, a_addr
    );
    kfree(b);

    println!("OK");
}

fn test_alignment() {
    print!("[Heap test 6] Alignment ... ");

    // All returned pointers must be ALLOC_ALIGN-aligned regardless of
    // the requested size.
    for size in [1, 3, 7, 13, 31, 64, 100, 255] {
        let ptr = kmalloc(size);
        assert!(!ptr.is_null(), "kmalloc({}) failed", size);
        assert_eq!(
            ptr as usize % ALLOC_ALIGN,
            0,
            "kmalloc({}) returned unaligned pointer {:#x}",
            size,
            ptr as usize
        );
        kfree(ptr);
    }

    println!("OK");
}

fn test_zero_alloc() {
    print!("[Heap test 7] Zero-size alloc ... ");

    let ptr = kmalloc(0);
    assert!(ptr.is_null(), "kmalloc(0) should return null");

    // kfree(null) should be a no-op, not crash
    kfree(core::ptr::null_mut());

    println!("OK");
}
