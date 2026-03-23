# Kernel Memory System - Quick Reference

## Target
i386 higher-half kernel, Multiboot2, QEMU. Must stay under 10 MB.

## Memory Layout (Virtual)
```
0x00000000–0x003FFFFF  Identity map (CLEARED after boot)
0xC0000000–0xC03FFFFF  Kernel image (1 page table, set by bootstrap.asm)
0xC0000800             GDT (GDTADDR)
0xC1000000–0xC1FFFFFF  Kernel heap (16 MB region, initially 128 KB mapped)
0xD0000000+            Available for dynamic mapping (used by VMM tests)
0xFFC00000–0xFFFFFFFF  Recursive page directory mapping (PDE[1023])
```

## File Structure

### Assembly (boot)
- `bootstrap.asm` - entry point, sets up stack, identity+higher-half paging, exports `page_directory`, `page_table1`, `stack_top`, `stack_bottom`, `clear_page1`
- `boot.asm` - higher_half_start, calls `rust_main`, `clear_page1` implementation

### Kernel entry
- `lib.rs` - `rust_main()` → `init()` → memory::init(), gdt::init(), interrupts::init(), shell::init_shell(). Declares `extern crate alloc`, registers `#[global_allocator]` (KernelAllocator), `#[alloc_error_handler]` lives in allocator.rs.

### Memory subsystem (`src/memory/`)
- `mod.rs` - `init()` orchestrator: physical → vmm → clear identity map → diagnose → vmm tests → heap init → heap tests → alloc tests (feature-gated)
- `define.rs` - PAGE_SIZE=4096, KERNEL_OFFSET=0xC0000000, heap region constants (KERNEL_HEAP_START=0xC1000000, KERNEL_HEAP_END=0xC2000000, KERNEL_HEAP_INITIAL_SIZE=128KB)
- `paging.rs` - Unified PageEntry type and PageDirectory for both levels
- `physical.rs` - FrameAllocator: bitmap-based, allocate/deallocate/specific frames, kernel+bitmap region protection
- `vmm.rs` - Virtual Memory Manager using recursive PDE[1023] mapping. map_page, unmap_page, translate, map_alloc, map_range, map_range_to, unmap_range. Full self-test suite.
- `heap.rs` - Kernel heap: linked-list free-list allocator. kmalloc, kfree, ksize, grow_heap, print_stats. Auto-grows by mapping new pages via VMM. Full self-test suite (7 tests).
- `allocator.rs` - GlobalAlloc trait implementation wrapping kmalloc/kfree. Zero-sized KernelAllocator struct. Fast path for align≤8, over-allocate+stash path for higher alignments. Alloc error handler (panics on OOM). Self-test suite behind `alloc_test` feature (6 tests: Box, Vec, String, vec!, large alloc, over-aligned).

### GDT (`src/gdt/`)
- `mod.rs` - 8 segments: null, kernel code/data/stack, user code/data/stack, TSS
- `define.rs` - selectors (0x08, 0x10, 0x18, 0x20|3, 0x28|3, 0x30|3, 0x38), KERNEL_VIRTUAL_BASE, GDTADDR, GDTSIZE
- `descriptor.rs` - SegmentDescriptor (8-byte packed)
- `tss.rs` - TssSegment, init with kernel stack, user/kernel mode selector switching

### Interrupts (`src/interrupts/`)
- `mod.rs` - init: bind handlers, PIC init, load IDT, configure IRQs
- `idt.rs` - IDT (256 entries), load_idt, configure_interrupts (keyboard enabled)
- `handlers.rs` - divide_by_zero, page_fault (reads CR2, decodes error code, kernel_panic), keyboard, double_fault, GPF, default. kernel_panic() with register dump + halt.
- `pic.rs` - 8259 PIC remap to 0x20/0x28, set_irq_state
- `interrupts.rs` - Interrupt enum (CPU exceptions 0-21, hardware IRQs 32-47, syscall 128)

### Multiboot (`src/multiboot2/`)
- `mod.rs` - MultibootInfo tag iterator
- `meminfo.rs` - Memory map parsing from multiboot, static buffer (MAX_MEMORY_ENTRIES=32)

## Key Constants
| Constant | Value | Location |
|---|---|---|
| KERNEL_OFFSET | 0xC0000000 | memory/define.rs |
| PAGE_SIZE | 4096 | memory/define.rs |
| KERNEL_HEAP_START | 0xC1000000 | memory/define.rs |
| KERNEL_HEAP_END | 0xC2000000 | memory/define.rs |
| KERNEL_HEAP_INITIAL_SIZE | 128 * 1024 | memory/define.rs |
| RECURSIVE_INDEX | 1023 | vmm.rs |
| PAGE_TABLES_VBASE | 0xFFC00000 | vmm.rs |
| ALLOC_ALIGN / KMALLOC_ALIGN | 8 | heap.rs / allocator.rs |

## Public APIs

### Physical (`physical.rs`)
```rust
FRAME_ALLOCATOR.allocate_frame() -> Result<PhysFrame, AllocationError>
FRAME_ALLOCATOR.deallocate_frame(frame) -> Result<(), AllocationError>
FRAME_ALLOCATOR.allocate_specific_frame(frame) -> Result<(), AllocationError>
```

### VMM (`vmm.rs`)
```rust
vmm::map_page(virt: VirtAddr, phys: PhysAddr, flags: PageFlags) -> Result<(), MapError>
vmm::map_alloc(virt: VirtAddr, flags: PageFlags) -> Result<PhysAddr, MapError>  // alloc frame + map
vmm::unmap_page(virt: VirtAddr) -> Result<PhysAddr, UnmapError>
vmm::translate(virt: VirtAddr) -> Option<PhysAddr>
vmm::is_mapped(virt: VirtAddr) -> bool
vmm::map_range(start, size, flags) -> Result<usize, MapError>  // multi-page, auto-alloc
vmm::map_range_to(virt, phys, size, flags) -> Result<usize, MapError>  // identity-style
vmm::unmap_range(start, size) -> usize  // returns pages freed
```

### Heap (`heap.rs`)
```rust
heap::kmalloc(size: usize) -> *mut u8       // null on failure, 8-byte aligned
heap::kfree(ptr: *mut u8)                   // null-safe, double-free detected
heap::ksize(ptr: *mut u8) -> usize          // 0 for null
heap::grow_heap(size: usize) -> Result<usize, MapError>  // pre-grow, page-aligned
heap::print_stats()                          // diagnostic output
```

### GlobalAlloc (`allocator.rs`)
```rust
// Registered as #[global_allocator] in lib.rs — enables:
//   alloc::boxed::Box, alloc::vec::Vec, alloc::string::String, etc.
//
// KernelAllocator is zero-sized; all state lives in heap.rs.
// align ≤ 8  → direct kmalloc/kfree (zero overhead)
// align > 8  → over-allocate, align, stash original ptr for dealloc
// size == 0  → returns non-null sentinel (align as *mut u8)
// OOM        → #[alloc_error_handler] panics with size/align info
```

## Cargo Features
| Feature | Effect |
|---|---|
| `verbose` | Extra logging during init (paging, frame allocator, VMM, GDT) |
| `gdt_test` | GDT structure, segment register, and TSS verification tests |
| `alloc_test` | GlobalAlloc self-tests: Box, Vec, String, vec!, large alloc, over-aligned alloc |

## Remaining Work
1. **User space memory** - user page tables, per-process address space, user-accessible mappings
2. **Demand paging / CoW** in page fault handler (currently panics on all faults)
3. **Proper locking** - replace `static mut` with spinlocks for SMP readiness

## Common Pitfalls
1. Virtual vs Physical: subtract KERNEL_OFFSET when writing to page tables
2. TLB: invlpg after PTE changes, full flush after PDE changes
3. Alignment: all page table addresses must be 4KB aligned
4. Identity map cleared: don't access low memory after clear_page1()
5. Recursive mapping: PDE[1023] is reserved, never map into it
6. Heap growth: auto-grows via vmm::map_range, but bounded by KERNEL_HEAP_END
7. GlobalAlloc alignment: over-aligned dealloc must recover the stashed original pointer — always pass the correct Layout to dealloc