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
    0xC1000000 : Kernel heap start (PLANNED)
    0xC2000000 : Kernel heap end   (PLANNED)

Physical Memory:
0x00100000 (1MB) : Kernel load address (KERNEL_LMA)
```

## File Structure

```
src/
├── lib.rs                 # Entry point, calls memory::init()
├── memory/
│   ├── mod.rs             # init(): PAGING global, diagnostic, physical allocator, clear identity map ✅
│   ├── define.rs          # PAGE_SIZE, KERNEL_OFFSET, page_directory extern ✅
│   ├── pageflags.rs       # PageFlags bitflags (BitOr, BitAnd, Not) ✅
│   ├── directory.rs       # PageDirectory + PageDirectoryEntry: COMPLETE ✅
│   ├── pagetable.rs       # PageTable + PageTableEntry: COMPLETE ✅
│   └── physical.rs        # FrameAllocator - COMPLETE ✅
├── multiboot2/
│   ├── mod.rs             # Multiboot2 parsing ✅
│   └── meminfo.rs         # Memory map from multiboot ✅
├── gdt/
│   ├── mod.rs             # GDT init + TSS ✅
│   ├── define.rs          # Segment selectors ✅
│   ├── descriptor.rs      # SegmentDescriptor ✅
│   └── tss.rs             # TssSegment ✅
├── boot.asm               # higher_half_start, clear_page1() ✅
└── bootstrap.asm          # start, setup_paging, stack, page tables
                           # Exports: page_directory, page_table1, stack_top, stack_bottom ✅
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
; bootstrap.asm - available via extern "C"
global page_directory      ; Page directory base (4KB aligned, phys addr via -KERNEL_OFFSET)
global page_table1         ; First page table (4KB aligned)
global stack_top           ; Kernel stack top
global stack_bottom        ; 16KB stack

; boot.asm
global clear_page1         ; Clears first PDE (removes 0-4MB identity map)
```

## Paging API (directory.rs / pagetable.rs)

```rust
// Global instance (memory/mod.rs)
pub static mut PAGING: Option<PageDirectory>
pub fn paging() -> &'static mut PageDirectory  // safe accessor

// PageDirectory methods
PageDirectory::new() -> Self                   // wraps the bootstrap page_directory symbol
pd.set_entry(index, phys_addr, flags)          // write a PDE (asserts alignment)
pd.clear_entry(index)                          // zero a PDE
pd.get_entry(index) -> *mut PageDirectoryEntry // raw pointer to entry
pd.physical_address() -> u32                   // virtual addr - KERNEL_OFFSET

// PageTable methods
PageTable::new(addr: *mut [u32; 1024]) -> Self
pt.set_entry(index, phys_addr, flags)
pt.clear_entry(index)
pt.get_entry(index) -> *mut PageTableEntry
pt.zero()                                      // clear all 1024 entries
pt.physical_address() -> u32

// PageFlags constants (pageflags.rs)
PageFlags::PRESENT | WRITABLE | USER | WRITE_THROUGH
         | CACHE_DISABLE | ACCESSED | DIRTY | HUGE_PAGE | GLOBAL
flags.value() -> u32
flags.is_present() / is_writable() / is_user() -> bool
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

## Diagnostic Utilities (memory/mod.rs)

```rust
// Called automatically during memory::init()
diagnose_page_directory()       // prints CR3, checks PDE[0] and PDE[768], lists all present PDEs
test_paging_infrastructure()    // unit-tests PDE/PTE ops and PageFlags, reads live page directory
```

## What's Missing

### Virtual Memory Manager (memory/virtual.rs - TODO)
```rust
pub struct VirtAddr(pub u32);
pub struct PhysAddr(pub u32);

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
// Planned region: 0xC1000000 - 0xC2000000 (16MB)
```

### Page Fault Handler (interrupt 14 - TODO)
```rust
// Read faulting address from CR2
// Print error and halt
```

## Common Pitfalls

1. **Virtual vs Physical**: Subtract `KERNEL_VIRTUAL_BASE` (or `KERNEL_OFFSET`) when writing to page tables
2. **TLB**: Must `invlpg` (or reload CR3) after modifying page table entries
3. **Alignment**: Page table addresses must be 4KB aligned — `set_entry` asserts this
4. **Identity map**: `clear_page1()` removes 0-4MB mapping — don't access low memory after boot
5. **get_entry pointer arithmetic**: Cast to `*mut Entry` before `.add(index)`, not to array pointer (wrong stride)

## Implementation Priority

1. Create `VirtAddr`/`PhysAddr` types and `map_page` / `unmap_page` / `translate` API
2. Add page fault handler (interrupt 14) reading CR2
3. Define heap region constants, implement bump allocator in `memory/heap.rs`
4. Upgrade to free-list heap allocator
5. Implement `GlobalAlloc` for Rust `alloc` crate
6. Enhance panic: red screen, register dump, CR2 on page fault