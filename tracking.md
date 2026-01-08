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
    0xC0100000+: Kernel heap (TODO - not defined)

Physical Memory:
0x00100000 (1MB) : Kernel load address (KERNEL_LMA in linker.ld)
```

## Current File Structure

```
src/
├── lib.rs              # Entry point, calls memory::init()
├── memory/
│   ├── mod.rs          # init() clears identity map, exposes PAGING global
│   ├── define.rs       # PAGE_SIZE=4096, KERNEL_OFFSET=0xC0000000
│   ├── pageflags.rs    # PageFlags bitflags (PRESENT, WRITABLE, USER, etc.)
│   ├── directory.rs    # PageDirectory, PageDirectoryEntry (read-only getters)
│   ├── pagetable.rs    # PageTable, PageTableEntry (read-only getters)
│   └── physical.rs     # FrameAllocator with bitmap, PhysFrame
├── gdt/
│   ├── mod.rs          # GDT init with kernel+user segments + TSS
│   ├── define.rs       # Selectors: KERNEL_CODE=0x08, USER_CODE=0x23, etc.
│   ├── descriptor.rs   # SegmentDescriptor struct
│   └── tss.rs          # TssSegment for ring transitions
├── boot.asm            # higher_half_start, clear_page1()
└── bootstrap.asm       # start, setup_paging (assembly), stack, page tables
```

## What Works

| Component | Status | Location |
|-----------|--------|----------|
| Multiboot2 boot | ✅ | bootstrap.asm |
| Higher-half paging (0xC0000000) | ✅ | bootstrap.asm::setup_paging |
| Identity map cleanup | ✅ | boot.asm::clear_page1 |
| Physical frame allocator | ✅ | physical.rs::FrameAllocator |
| Page flags | ✅ | pageflags.rs::PageFlags |
| GDT with ring 0/3 segments | ✅ | gdt/mod.rs |
| TSS for stack switching | ✅ | gdt/tss.rs |
| Basic panic handler | ✅ | lib.rs (print + loop) |

## What's Missing (Implementation TODO)

### 1. Virtual Memory Manager (CRITICAL)
Need in `memory/` module:
```rust
// Virtual address wrapper
pub struct VirtAddr(u32);

// Core mapping functions
pub fn map_page(virt: VirtAddr, phys: PhysFrame, flags: PageFlags) -> Result<(), MapError>;
pub fn unmap_page(virt: VirtAddr) -> Result<PhysFrame, UnmapError>;
pub fn translate(virt: VirtAddr) -> Option<PhysAddr>;

// TLB management
pub fn flush_tlb_entry(virt: VirtAddr); // invlpg instruction
pub fn flush_tlb_all();                  // reload CR3
```

### 2. Dynamic Page Table Allocation (CRITICAL)
The `set_entry()` methods in `directory.rs` and `pagetable.rs` are **commented out**. Need:
- Uncomment and fix `set_entry()` methods
- Allocate new page tables using `FrameAllocator`
- Track which page tables are allocated

### 3. Kernel Heap Allocator (CRITICAL)
Need new file `memory/heap.rs`:
```rust
// Simple interface
pub fn kmalloc(size: usize) -> *mut u8;
pub fn kfree(ptr: *mut u8);
pub fn ksize(ptr: *mut u8) -> usize;

// For Rust alloc crate integration
impl GlobalAlloc for KernelAllocator { ... }
```
Suggested heap region: `0xC1000000` to `0xC2000000` (16MB)

### 4. Page Fault Handler (HIGH)
In interrupts module, add handler for interrupt 14:
```rust
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    let fault_addr: u32;
    unsafe { asm!("mov {}, cr2", out(reg) fault_addr); }
    panic!("PAGE FAULT at {:#x}, error: {:#x}", fault_addr, error_code);
}
```

### 5. User Space Memory (LOW PRIORITY)
- Separate page directory per process
- User-accessible page mappings (USER flag)
- Syscall interface for memory operations

## Key Constants Reference

```rust
// From gdt/define.rs
KERNEL_CODE_SELECTOR  = 0x08
KERNEL_DATA_SELECTOR  = 0x10
KERNEL_STACK_SELECTOR = 0x18
USER_CODE_SELECTOR    = 0x23  // 0x20 | 3
USER_DATA_SELECTOR    = 0x2B  // 0x28 | 3
USER_STACK_SELECTOR   = 0x33  // 0x30 | 3
TSS_SELECTOR          = 0x38
KERNEL_VIRTUAL_BASE   = 0xC0000000

// From memory/define.rs
PAGE_SIZE             = 4096
PAGE_TABLE_ENTRIES    = 1024
PAGE_DIRECTORY_ENTRIES = 1024
KERNEL_OFFSET         = 0xC0000000
```

## Assembly Interfaces

```nasm
; Defined in bootstrap.asm (available to Rust via extern)
global page_directory      ; Page directory base (4KB aligned)
global stack_top           ; Kernel stack top
global stack_bottom        ; Kernel stack bottom (16KB stack)

; Defined in boot.asm
global clear_page1         ; Clears first PDE (removes identity map)
```

## Physical Memory Detection

Multiboot2 provides memory map via `multiboot2::meminfo::MemoryInfoEntry`:
```rust
struct MemoryInfoEntry {
    base_addr: u64,
    length: u64,
    typee: u32,  // 1 = available, other = reserved
}
```
Used by `FrameAllocator::new()` to mark available frames.

## Implementation Priority Order

1. **Uncomment `set_entry()` methods** in directory.rs/pagetable.rs
2. **Create `map_page()` / `unmap_page()` API** using FrameAllocator
3. **Add page fault handler** to IDT (interrupt 14)
4. **Define heap region** and implement bump allocator
5. **Upgrade to proper heap** (free list or buddy allocator)
6. **Implement GlobalAlloc** for Rust `alloc` crate
7. **Enhance panic handler** with register dump

## Testing Checklist

- [ ] Can allocate physical frame via `FrameAllocator`
- [ ] Can map new virtual page to physical frame
- [ ] Can unmap page and free frame
- [ ] Page fault triggers handler (not triple fault)
- [ ] `kmalloc()` returns valid pointer
- [ ] `kfree()` doesn't corrupt heap
- [ ] Multiple alloc/free cycles work
- [ ] Kernel stays under 10MB total

## Common Pitfalls

1. **Virtual vs Physical addresses**: Always subtract `KERNEL_VIRTUAL_BASE` when writing to page tables (they need physical addresses)
2. **TLB caching**: Must `invlpg` after modifying page tables
3. **Page alignment**: All page table/directory addresses must be 4KB aligned
4. **Identity map removal**: `clear_page1()` removes 0-4MB identity map; don't access low memory after this
5. **Bitmap initialization**: FrameAllocator marks all frames used initially, then frees available regions