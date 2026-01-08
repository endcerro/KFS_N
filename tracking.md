# i386 Kernel Memory System - Project Context

## Target Architecture
- **CPU**: i386 (32-bit x86)
- **Boot**: Multiboot2 via GRUB
- **Kernel Base**: Higher-half at `0xC0000000`
- **Toolchain**: Rust (`no_std`) + NASM assembly

## Memory Layout

```
Virtual Address Space:
0x00000000 - 0xBFFFFFFF : User Space (3GB) - NOT YET IMPLEMENTED
0xC0000000 - 0xFFFFFFFF : Kernel Space (1GB)
    0xC0000000 : Kernel code/data start (_kernel_start)
    0xC0000800 : GDT location (GDTADDR)
    _kernel_end + PAGE_ALIGN : Frame allocator bitmap

Physical Memory:
0x00100000 (1MB) : Kernel load address (KERNEL_LMA)
```

## File Structure

```
src/
├── lib.rs                 # Entry point, calls memory::init()
├── memory/
│   ├── mod.rs             # init(): physical allocator + clear identity map
│   ├── define.rs          # PAGE_SIZE, KERNEL_OFFSET constants
│   ├── pageflags.rs       # PageFlags bitflags
│   ├── directory.rs       # PageDirectory (set_entry commented out)
│   ├── pagetable.rs       # PageTable (set_entry commented out)
│   └── physical.rs        # FrameAllocator - COMPLETE
├── multiboot2/
│   ├── mod.rs             # Multiboot2 parsing
│   └── meminfo.rs         # Memory map from multiboot
├── gdt/
│   ├── mod.rs             # GDT init + TSS
│   ├── define.rs          # Segment selectors
│   ├── descriptor.rs      # SegmentDescriptor
│   └── tss.rs             # TssSegment
├── boot.asm               # higher_half_start, clear_page1()
└── bootstrap.asm          # start, setup_paging, stack, page tables
```

## Key Constants

```rust
// gdt/define.rs
KERNEL_CODE_SELECTOR  = 0x08
KERNEL_DATA_SELECTOR  = 0x10
KERNEL_STACK_SELECTOR = 0x18
USER_CODE_SELECTOR    = 0x23  // 0x20 | 3
USER_DATA_SELECTOR    = 0x2B  // 0x28 | 3
USER_STACK_SELECTOR   = 0x33  // 0x30 | 3
TSS_SELECTOR          = 0x38
KERNEL_VIRTUAL_BASE   = 0xC0000000

// memory/define.rs
PAGE_SIZE              = 4096
PAGE_TABLE_ENTRIES     = 1024
PAGE_DIRECTORY_ENTRIES = 1024
KERNEL_OFFSET          = 0xC0000000
```

## Assembly Interfaces

```nasm
; bootstrap.asm - available via extern
global page_directory      ; Page directory base (4KB aligned)
global stack_top           ; Kernel stack top
global stack_bottom        ; 16KB stack

; boot.asm
global clear_page1         ; Clears first PDE (removes identity map)
```

## Physical Frame Allocator API (physical.rs)

```rust
// Global instance
pub static mut FRAME_ALLOCATOR: Option<FrameAllocator>

// Frame operations
PhysFrame::containing_address(addr: usize) -> PhysFrame
PhysFrame::start_address(&self) -> usize

// Allocator methods
allocate_frame() -> Result<PhysFrame, AllocationError>
allocate_specific_frame(frame) -> Result<(), AllocationError>
deallocate_frame(frame) -> Result<(), AllocationError>
total_frames() / used_frames() / free_frames() -> usize
```

## What's Missing

### Virtual Memory Manager (memory/virtual.rs - TODO)
```rust
pub fn map_page(virt: VirtAddr, phys: PhysFrame, flags: PageFlags) -> Result<(), MapError>;
pub fn unmap_page(virt: VirtAddr) -> Result<PhysFrame, UnmapError>;
pub fn translate(virt: VirtAddr) -> Option<PhysAddr>;
pub fn flush_tlb_entry(virt: VirtAddr);  // invlpg
pub fn flush_tlb_all();                   // reload CR3
```

### Heap Allocator (memory/heap.rs - TODO)
```rust
pub fn kmalloc(size: usize) -> *mut u8;
pub fn kfree(ptr: *mut u8);
pub fn ksize(ptr: *mut u8) -> usize;
// Suggested region: 0xC1000000 - 0xC2000000 (16MB)
```

### Page Fault Handler (interrupt 14 - TODO)
```rust
// Read faulting address from CR2
// Print error and halt
```

## Common Pitfalls

1. **Virtual vs Physical**: Subtract `KERNEL_VIRTUAL_BASE` when writing to page tables
2. **TLB**: Must `invlpg` after modifying page tables
3. **Alignment**: Page table addresses must be 4KB aligned
4. **Identity map**: `clear_page1()` removes 0-4MB mapping - don't access low memory after

## Implementation Priority

1. Uncomment `set_entry()` in directory.rs/pagetable.rs
2. Create `map_page()` / `unmap_page()` API
3. Add page fault handler (interrupt 14)
4. Define heap region, implement bump allocator
5. Upgrade to free-list heap allocator
6. Implement `GlobalAlloc` for Rust `alloc` crate