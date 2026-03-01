// memory/heap.rs — Kernel heap region management
//
// This module is responsible for:
//   1. Mapping the initial kernel heap pages at boot
//   2. Growing the heap on demand by mapping more pages
//   3. Tracking the current heap watermark (how much is mapped)
//
// The actual allocator (free-list / bump / etc.) lives on top of this
// and will be added in a later phase.
//
// Heap layout:
//   KERNEL_HEAP_START (0xC100_0000) ─────── mapped, usable
//        ...
//   heap_end (grows upward)  ─────────────── current watermark
//        ...
//   KERNEL_HEAP_END   (0xC200_0000) ─────── maximum, unmapped until needed

use super::define::{
    KERNEL_HEAP_START, KERNEL_HEAP_END, KERNEL_HEAP_INITIAL_SIZE, PAGE_SIZE,
};
use super::vmm::{self, VirtAddr, MapError};
use super::pageflags::PageFlags;

/// Current end of the mapped heap region.  Everything in
/// [KERNEL_HEAP_START .. heap_mapped_end) is backed by physical frames.
static mut HEAP_MAPPED_END: usize = KERNEL_HEAP_START;

/// Initialise the kernel heap by pre-mapping KERNEL_HEAP_INITIAL_SIZE
/// bytes starting at KERNEL_HEAP_START.
///
/// Must be called after vmm::init() and the physical frame allocator.
pub fn init() {
    let initial = KERNEL_HEAP_INITIAL_SIZE;
    assert!(initial <= KERNEL_HEAP_END - KERNEL_HEAP_START,
        "Initial heap size exceeds heap region");
    assert!(initial % PAGE_SIZE == 0,
        "Initial heap size must be page-aligned");

    println!("Heap: mapping initial region {:#x}..{:#x} ({} KB)",
        KERNEL_HEAP_START, KERNEL_HEAP_START + initial, initial / 1024);

    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;
    let pages = vmm::map_range(VirtAddr::new(KERNEL_HEAP_START as u32), initial, flags)
        .expect("Heap: failed to map initial region");

    unsafe {
        HEAP_MAPPED_END = KERNEL_HEAP_START + initial;
    }

    println!("Heap: {} pages mapped, region ready at {:#x}..{:#x}",
        pages, KERNEL_HEAP_START, KERNEL_HEAP_START + initial);
}

/// Grow the mapped heap region by `size` bytes (must be page-aligned).
///
/// Returns the start address of the newly mapped region, or an error
/// if we've hit the ceiling or run out of physical frames.
pub fn grow_heap(size: usize) -> Result<usize, MapError> {
    assert!(size % PAGE_SIZE == 0, "grow_heap: size must be page-aligned");

    let current_end = unsafe { HEAP_MAPPED_END };
    let new_end = current_end + size;

    if new_end > KERNEL_HEAP_END {
        println!("Heap: cannot grow past {:#x} (requested {:#x})", KERNEL_HEAP_END, new_end);
        return Err(MapError::InvalidAddress);
    }

    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;
    vmm::map_range(VirtAddr::new(current_end as u32), size, flags)?;

    unsafe {
        HEAP_MAPPED_END = new_end;
    }

    #[cfg(feature = "verbose")]
    println!("Heap: grew to {:#x} (+{} KB)", new_end, size / 1024);

    Ok(current_end)
}

/// Returns the start of the usable heap region.
pub fn heap_start() -> usize {
    KERNEL_HEAP_START
}

/// Returns the current end of the *mapped* heap region.
/// Everything in [heap_start() .. heap_mapped_end()) is safe to access.
pub fn heap_mapped_end() -> usize {
    unsafe { HEAP_MAPPED_END }
}

/// Returns the maximum possible heap end address.
pub fn heap_max() -> usize {
    KERNEL_HEAP_END
}

/// Returns how many bytes are currently mapped for the heap.
pub fn heap_mapped_size() -> usize {
    unsafe { HEAP_MAPPED_END - KERNEL_HEAP_START }
}

/// Print heap region statistics.
pub fn print_stats() {
    let mapped = heap_mapped_size();
    let max = KERNEL_HEAP_END - KERNEL_HEAP_START;
    println!("Kernel Heap:");
    println!("  Region:    {:#x}..{:#x} ({} KB max)",
        KERNEL_HEAP_START, KERNEL_HEAP_END, max / 1024);
    println!("  Mapped:    {:#x}..{:#x} ({} KB)",
        KERNEL_HEAP_START, heap_mapped_end(), mapped / 1024);
    println!("  Remaining: {} KB unmapped",
        (max - mapped) / 1024);
}