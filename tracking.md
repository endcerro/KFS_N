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
0xC0000000 - 0xFFBFFFFF : Kernel Space (~1GB minus recursive region)
    0xC0000000 : Kernel code/data start (_kernel_start)
    0xC0000800 : GDT location (GDTADDR)
    _kernel_end + PAGE_ALIGN : Frame allocator bitmap (virtual = phys + KERNEL_OFFSET)
    0xC1000000 : Kernel heap start (PLANNED)
    0xC2000000 : Kernel heap end   (PLANNED)
0xFFC00000 - 0xFFFFFFFF : Recursive page directory mapping (4MB, reserved)
    0xFFC00000 + N*0x1000 : Virtual access to page table N
    0xFFFFF000            : Virtual access to the page directory itself

Physical Memory:
0x00100000 (1MB) : Kernel load address (KERNEL_LMA)
```

## File Structure

```
src/
├── lib.rs                 # Entry point, calls memory::init()
├── memory/
│   ├── mod.rs             # init(): PAGING global, diagnostic, physical allocator, VMM init ✅
│   ├── define.rs          # PAGE_SIZE, KERNEL_OFFSET, page_directory extern ✅
│   ├── pageflags.rs       # PageFlags bitflags (NONE, PRESENT..GLOBAL, BitOr/BitAnd/Not) ✅
│   ├── directory.rs       # PageDirectory + PageDirectoryEntry ✅
│   ├── pagetable.rs       # PageTable + PageTableEntry ✅
│   ├── physical.rs        # FrameAllocator (bitmap uses virtual address) ✅
│   └── vmm.rs             # Virtual Memory Manager — recursive mapping, map/unmap/translate ✅
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

// memory/vmm.rs
RECURSIVE_INDEX        = 1023       // PDE[1023] → page directory itself
PAGE_TABLES_VBASE      = 0xFFC00000 // page table N at VBASE + N * 0x1000
PAGE_DIR_VIRT          = 0xFFFFF000 // page directory at VBASE + 1023 * 0x1000
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
PageFlags::NONE | PRESENT | WRITABLE | USER | WRITE_THROUGH
         | CACHE_DISABLE | ACCESSED | DIRTY | HUGE_PAGE | GLOBAL
PageFlags::from_raw(u32) -> PageFlags
flags.value() -> u32
flags.is_present() / is_writable() / is_user() -> bool
```

## Virtual Memory Manager API (vmm.rs)

```rust
// Address types
pub struct VirtAddr(pub u32);   // .pde_index(), .pte_index(), .page_offset(), .is_kernel()
pub struct PhysAddr(pub u32);   // .is_page_aligned()

// Initialisation — installs recursive mapping at PDE[1023]
pub fn init();

// Core operations
pub fn map_page(virt: VirtAddr, phys: PhysAddr, flags: PageFlags) -> Result<(), MapError>;
pub fn map_alloc(virt: VirtAddr, flags: PageFlags) -> Result<PhysAddr, MapError>;  // allocates frame + maps
pub fn unmap_page(virt: VirtAddr) -> Result<PhysAddr, UnmapError>;
pub fn translate(virt: VirtAddr) -> Option<PhysAddr>;
pub fn is_mapped(virt: VirtAddr) -> bool;

// TLB management
pub fn flush_tlb_entry(virt: VirtAddr);  // invlpg
pub fn flush_tlb_all();                   // reload CR3

// Self-test (called automatically from memory::init())
pub fn test_virtual_memory();
```

### How recursive mapping works
PDE[1023] points to the page directory itself. When the MMU resolves addresses
in `0xFFC00000–0xFFFFFFFF`, it re-enters the PD as a page table, giving:
- Page table N at `0xFFC00000 + N * 0x1000`
- Page directory at `0xFFFFF000`
This lets `map_page`/`unmap_page` read/write any PDE or PTE through normal
virtual memory dereferences, with no temporary mappings needed.

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

**Note**: The bitmap pointer stores the *virtual* address (`phys + KERNEL_OFFSET`),
not the physical address. This is critical because the identity map is removed
after boot by `clear_page1()`.

## Diagnostic Utilities (memory/mod.rs)

```rust
// Called automatically during memory::init()
diagnose_page_directory()       // prints CR3, checks PDE[0] and PDE[768], lists all present PDEs
test_paging_infrastructure()    // unit-tests PDE/PTE ops and PageFlags, reads live page directory
vmm::test_virtual_memory()      // 5-test VMM suite: recursive reads, map/write/unmap, translate, multi-page, error cases
```

## What's Missing

### Heap Allocator (memory/heap.rs - TODO)
```rust
pub fn kmalloc(size: usize) -> *mut u8;
pub fn kfree(ptr: *mut u8);
pub fn ksize(ptr: *mut u8) -> usize;
// Planned region: 0xC1000000 - 0xC2000000 (16MB)
// Implement GlobalAlloc trait for Rust alloc crate
```

### Page Fault Handler (interrupt 14 - TODO)
```rust
// Read faulting address from CR2
// Print error code breakdown (present/write/user/reserved/fetch)
// Print registers and halt
```

### User Space Memory (TODO)
```rust
// User page tables (per-process)
// Syscall mechanism for user memory requests
// User heap allocator
```

## Common Pitfalls

1. **Virtual vs Physical**: Subtract `KERNEL_VIRTUAL_BASE` (or `KERNEL_OFFSET`) when writing to page tables
2. **TLB**: Must `invlpg` (or reload CR3) after modifying page table entries
3. **Alignment**: Page table addresses must be 4KB aligned — `set_entry` asserts this
4. **Identity map**: `clear_page1()` removes 0-4MB mapping — don't store physical addresses in pointers that outlive boot
5. **get_entry pointer arithmetic**: Cast to `*mut Entry` before `.add(index)`, not to array pointer (wrong stride)
6. **Bitmap pointer**: Must use virtual address (phys + KERNEL_OFFSET), not physical — identity map gone after boot
7. **Recursive mapping**: PDE[1023] is reserved — never map user/kernel pages into the last 4MB of virtual space

## Implementation Priority

1. ~~Create `VirtAddr`/`PhysAddr` types and `map_page` / `unmap_page` / `translate` API~~ ✅
2. Add page fault handler (interrupt 14) reading CR2
3. Define heap region constants, implement bump allocator in `memory/heap.rs`
4. Upgrade to free-list heap allocator
5. Implement `GlobalAlloc` for Rust `alloc` crate
6. Enhance panic: red screen, register dump, CR2 on page fault