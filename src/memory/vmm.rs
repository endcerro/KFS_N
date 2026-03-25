// memory/vmm.rs - Virtual Memory Manager
//
// Uses **recursive page directory mapping** to access page tables without
// needing a separate temporary-mapping mechanism.
//
// How it works:
//   PDE[1023] points back to the page directory itself.  When the MMU
//   walks a virtual address whose PDE index is 1023, it re-enters the
//   page directory as if it were a page table.  This gives us:
//
//     - Page directory accessible at   0xFFFFF000
//       (PDE index 1023 -> PD, PTE index 1023 -> PD[1023] -> PD again)
//
//     - Page table N accessible at     0xFFC00000 + N * 0x1000
//       (PDE index 1023 -> PD, PTE index N -> PD[N] -> page table N)
//
//   So to read/write PDE[i]:  *(0xFFFFF000 + i*4)
//   To read/write PT_N[j]:    *((0xFFC00000 + N*0x1000) + j*4)
//
// TLB discipline:
//   - After creating a new page table (setting a PDE), flush_tlb_all()
//     so the recursive mapping reflects the new PDE.
//   - After setting a PTE, invlpg the target virtual address.
//
// All PDE/PTE access goes through the unified PageEntry type and
// PageFlags - no raw bitmask constants in this file.

use super::define::{KERNEL_OFFSET, PAGE_SIZE};
use super::pageflags::PageFlags;
use super::paging::PageEntry;
use super::physical::{PhysFrame, FRAME_ALLOCATOR};
use crate::dbg_println;
use crate::m_print;
use crate::m_println;
// ---------------------------------------------------------------------------
// Address wrapper types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub u32);

impl VirtAddr {
    #[inline]
    pub fn new(addr: u32) -> Self {
        VirtAddr(addr)
    }
    // Bits [31:22] - selects one of 1024 page directory entries
    #[inline]
    pub fn pde_index(&self) -> usize {
        (self.0 >> 22) as usize
    }
    // Bits [21:12] - selects one of 1024 page table entries
    #[inline]
    pub fn pte_index(&self) -> usize {
        ((self.0 >> 12) & 0x3FF) as usize
    }
    // Bits [11:0] - byte offset within the 4 KB page
    #[inline]
    pub fn page_offset(&self) -> u32 {
        self.0 & 0xFFF
    }
    #[inline]
    pub fn is_page_aligned(&self) -> bool {
        self.0 & 0xFFF == 0
    }
    // Is this address in the kernel half (>= 0xC0000000)?
    #[inline]
    pub fn is_kernel(&self) -> bool {
        self.0 >= KERNEL_OFFSET as u32
    }
}

impl PhysAddr {
    #[inline]
    pub fn new(addr: u32) -> Self {
        PhysAddr(addr)
    }
    #[inline]
    pub fn is_page_aligned(&self) -> bool {
        self.0 & 0xFFF == 0
    }
}

// ---------------------------------------------------------------------------
// Recursive mapping constants
// ---------------------------------------------------------------------------

// We sacrifice the last 4 MB of virtual space (0xFFC00000-0xFFFFFFFF)
// for the recursive mapping.  This region is kernel-only.
const RECURSIVE_INDEX: usize = 1023;

// Base virtual address where page table N is mapped:
//   page_table_virt(N) = PAGE_TABLES_VBASE + N * PAGE_SIZE
const PAGE_TABLES_VBASE: u32 = 0xFFC00000;

// Virtual address of the page directory itself (= page_table_virt(1023))
const PAGE_DIR_VIRT: u32 = PAGE_TABLES_VBASE + (RECURSIVE_INDEX as u32) * PAGE_SIZE as u32;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum MapError {
    FrameAllocationFailed,
    AlreadyMapped,
    InvalidAddress,
}

#[derive(Debug)]
pub enum UnmapError {
    NotMapped,
    PageTableNotPresent,
}

/// Errors returned by vmalloc / vfree.
#[derive(Debug)]
pub enum VmError {
    /// size argument was zero.
    ZeroSize,
    /// Address or range overlaps the recursive-mapping region (0xFFC00000+).
    /// That region is the live page directory — it cannot be remapped.
    RecursiveRegion,
    /// No free physical frames available.
    OutOfMemory,
    /// One or more pages in the requested range are already mapped.
    AlreadyMapped,
}

// ---------------------------------------------------------------------------
// Internal helper: round an address up to the next page boundary.
// If already aligned, the value is unchanged.
// ---------------------------------------------------------------------------
#[inline]
fn page_align_up(addr: u32) -> u32 {
    (addr.saturating_add(PAGE_SIZE as u32 - 1)) & !(PAGE_SIZE as u32 - 1)
}

// ---------------------------------------------------------------------------
// vmalloc / vfree / vsize
//
// Design:
//   - Any virtual address is accepted; addr is rounded UP to the nearest
//     page boundary before use.
//   - The only hard block is the recursive-mapping region (0xFFC00000+)
//     which is a hardware constraint — PDE[1023] is the page directory
//     itself and cannot be remapped.
//   - No bookkeeping table.  The caller owns (addr, size), mirroring
//     POSIX mmap/munmap.  vsize walks the live page tables directly.
// ---------------------------------------------------------------------------

/// Map `size` bytes starting at `addr` (rounded up to page boundary).
///
/// Returns `(aligned_addr, pages_mapped)` so the caller always knows
/// the actual base that was mapped even if rounding occurred.
pub fn vmalloc(addr: u32, size: usize) -> Result<(u32, usize), VmError> {
    if size == 0 {
        return Err(VmError::ZeroSize);
    }

    let aligned_addr = page_align_up(addr);
    let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
    let byte_size = pages * PAGE_SIZE;

    // Only hard constraint: must not touch the recursive-mapping region.
    if (aligned_addr as usize).saturating_add(byte_size) > 0xFFC0_0000 {
        return Err(VmError::RecursiveRegion);
    }

    map_range(
        VirtAddr::new(aligned_addr),
        byte_size,
        PageFlags::PRESENT | PageFlags::WRITABLE,
    )
    .map_err(|e| match e {
        MapError::FrameAllocationFailed => VmError::OutOfMemory,
        MapError::AlreadyMapped => VmError::AlreadyMapped,
        MapError::InvalidAddress => VmError::RecursiveRegion,
    })?;

    Ok((aligned_addr, pages))
}

/// Unmap `size` bytes starting at `addr` (rounded up to match vmalloc).
///
/// Returns the number of pages actually freed.
pub fn vfree(addr: u32, size: usize) -> Result<usize, VmError> {
    if size == 0 {
        return Err(VmError::ZeroSize);
    }

    let aligned_addr = page_align_up(addr);
    let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
    let byte_size = pages * PAGE_SIZE;

    let freed = unmap_range(VirtAddr::new(aligned_addr), byte_size);
    Ok(freed)
}

/// Count consecutive mapped pages from `addr` (rounded up to page boundary).
///
/// Walks the live page tables — the page tables are the ground truth.
/// Returns the number of mapped bytes, or 0 if the address is not mapped.
pub fn vsize(addr: u32) -> usize {
    let aligned_addr = page_align_up(addr);
    let mut count: usize = 0;

    loop {
        let cur = (aligned_addr as usize).saturating_add(count * PAGE_SIZE);
        if cur >= 0xFFC0_0000 {
            break;
        }
        if !is_mapped(VirtAddr::new(cur as u32)) {
            break;
        }
        count += 1;
    }

    count * PAGE_SIZE
}

// ---------------------------------------------------------------------------
// TLB helpers
// ---------------------------------------------------------------------------

// Invalidate a single TLB entry.
#[inline]
pub fn flush_tlb_entry(virt: VirtAddr) {
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) virt.0, options(nostack, preserves_flags));
    }
}

// Flush the entire TLB by reloading CR3.
#[inline]
pub fn flush_tlb_all() {
    unsafe {
        let cr3: u32;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack));
        core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack));
    }
}

// ---------------------------------------------------------------------------
// Typed PDE / PTE access via recursive mapping
//
// PageEntry is #[repr(C)] wrapping a single u32 (size = 4, align = 4),
// so ptr.add(index) correctly strides by 4 bytes - matching the
// hardware layout of 1024 × 4-byte entries per page table.
// ---------------------------------------------------------------------------

// Read PDE[index] through the recursive mapping.
#[inline]
unsafe fn read_pde(index: usize) -> PageEntry {
    let ptr = PAGE_DIR_VIRT as *const PageEntry;
    ptr.add(index).read_volatile()
}

// Write PDE[index] through the recursive mapping.
#[inline]
unsafe fn write_pde(index: usize, entry: PageEntry) {
    let ptr = PAGE_DIR_VIRT as *mut PageEntry;
    ptr.add(index).write_volatile(entry);
}

// Read PTE[pte_index] inside page table `pde_index`.
#[inline]
unsafe fn read_pte(pde_index: usize, pte_index: usize) -> PageEntry {
    let pt_base = (PAGE_TABLES_VBASE + pde_index as u32 * PAGE_SIZE as u32) as *const PageEntry;
    pt_base.add(pte_index).read_volatile()
}

// Write PTE[pte_index] inside page table `pde_index`.
#[inline]
unsafe fn write_pte(pde_index: usize, pte_index: usize, entry: PageEntry) {
    let pt_base = (PAGE_TABLES_VBASE + pde_index as u32 * PAGE_SIZE as u32) as *mut PageEntry;
    pt_base.add(pte_index).write_volatile(entry);
}

// ---------------------------------------------------------------------------
// Initialisation
// ---------------------------------------------------------------------------

// Set up recursive page directory mapping at PDE[1023].
//
// Must be called once, early in boot, after `PAGING` is initialised.
// After this, all map/unmap/translate operations use the recursive
// mapping and do not need the PageDirectory wrapper.
pub fn init() {
    unsafe {
        let pd = super::PAGING.as_mut().expect("PAGING not initialised");
        let pd_phys = pd.physical_address();

        pd.set_entry(
            RECURSIVE_INDEX,
            pd_phys,
            PageFlags::PRESENT | PageFlags::WRITABLE,
        );

        flush_tlb_all();
    }

    dbg_println!(
        "VMM: recursive mapping installed at PDE[{}]",
        RECURSIVE_INDEX
    );
}

// ---------------------------------------------------------------------------
// Core API
// ---------------------------------------------------------------------------

// Map a single 4 KB virtual page to a physical frame.
//
// If the page directory entry for this region has no page table yet,
// a new frame is allocated and zeroed automatically.
//
// `flags` are applied to the PTE.  The PDE is set to
// PRESENT | WRITABLE (and USER if `flags` includes USER), which is the
// standard "permissive PDE, restrictive PTE" policy.
pub fn map_page(virt: VirtAddr, phys: PhysAddr, flags: PageFlags) -> Result<(), MapError> {
    assert!(
        virt.is_page_aligned(),
        "virt addr {:#x} not page-aligned",
        virt.0
    );
    assert!(
        phys.is_page_aligned(),
        "phys addr {:#x} not page-aligned",
        phys.0
    );

    let pde_idx = virt.pde_index();
    let pte_idx = virt.pte_index();

    // The recursive slot is reserved - never map into it
    if pde_idx == RECURSIVE_INDEX {
        return Err(MapError::InvalidAddress);
    }

    unsafe {
        // Step 1: ensure a page table exists for this PDE
        let pde = read_pde(pde_idx);

        if !pde.present() {
            // No page table yet - allocate a physical frame for one
            let frame = alloc_frame()?;
            let pt_phys = frame.start_address() as u32;

            // PDE flags: permissive - the PTE is the real gatekeeper.
            let mut pde_flags = PageFlags::PRESENT | PageFlags::WRITABLE;
            if flags.is_user() {
                pde_flags = pde_flags | PageFlags::USER;
            }

            write_pde(pde_idx, PageEntry::new(pt_phys, pde_flags));

            // Full TLB flush so the recursive mapping exposes the new PT
            flush_tlb_all();

            // Zero the fresh page table - all 1024 PTEs become non-present
            let pt_virt = (PAGE_TABLES_VBASE + pde_idx as u32 * PAGE_SIZE as u32) as *mut u8;
            core::ptr::write_bytes(pt_virt, 0, PAGE_SIZE);

            dbg_println!(
                "VMM: allocated PT frame {:#x} for PDE[{}]",
                pt_phys,
                pde_idx
            );
        } else if flags.is_user() && !pde.user() {
            // Page table exists but PDE lacks USER - promote it
            let promoted = PageEntry::new(pde.address(), pde.flags() | PageFlags::USER);
            write_pde(pde_idx, promoted);
            flush_tlb_all();
        }

        // Step 2: set the PTE
        let pte = read_pte(pde_idx, pte_idx);
        if pte.present() {
            return Err(MapError::AlreadyMapped);
        }

        write_pte(pde_idx, pte_idx, PageEntry::new(phys.0, flags));
        flush_tlb_entry(virt);
    }

    dbg_println!(
        "VMM: mapped virt {:#x} -> phys {:#x} (flags {})",
        virt.0,
        phys.0,
        flags
    );

    Ok(())
}

// Convenience: allocate a physical frame *and* map it at `virt`.
pub fn map_alloc(virt: VirtAddr, flags: PageFlags) -> Result<PhysAddr, MapError> {
    let frame = alloc_frame()?;
    let phys = PhysAddr::new(frame.start_address() as u32);
    map_page(virt, phys, flags)?;
    Ok(phys)
}

// Unmap a single 4 KB virtual page.
//
// Returns the physical address the page was mapped to.
pub fn unmap_page(virt: VirtAddr) -> Result<PhysAddr, UnmapError> {
    let pde_idx = virt.pde_index();
    let pte_idx = virt.pte_index();

    unsafe {
        let pde = read_pde(pde_idx);
        if !pde.present() {
            return Err(UnmapError::PageTableNotPresent);
        }

        let pte = read_pte(pde_idx, pte_idx);
        if !pte.present() {
            return Err(UnmapError::NotMapped);
        }

        let phys = PhysAddr::new(pte.address());

        write_pte(pde_idx, pte_idx, PageEntry::empty());
        flush_tlb_entry(virt);

        dbg_println!("VMM: unmapped virt {:#x} (was phys {:#x})", virt.0, phys.0);

        Ok(phys)
    }
}

// Translate a virtual address to its physical address by walking
// PDE -> PTE.  Returns `None` if any level is not present.
pub fn translate(virt: VirtAddr) -> Option<PhysAddr> {
    let pde_idx = virt.pde_index();
    let pte_idx = virt.pte_index();

    unsafe {
        let pde = read_pde(pde_idx);
        if !pde.present() {
            return None;
        }

        let pte = read_pte(pde_idx, pte_idx);
        if !pte.present() {
            return None;
        }

        Some(PhysAddr::new(pte.address() | virt.page_offset()))
    }
}

// Quick predicate - is this virtual page currently mapped?
pub fn is_mapped(virt: VirtAddr) -> bool {
    translate(VirtAddr::new(virt.0 & !0xFFF)).is_some()
}

// ---------------------------------------------------------------------------
// Range operations
// ---------------------------------------------------------------------------

// Map a contiguous range of virtual pages, allocating a fresh physical
// frame for each one.
pub fn map_range(start: VirtAddr, size: usize, flags: PageFlags) -> Result<usize, MapError> {
    assert!(
        start.is_page_aligned(),
        "map_range: start {:#x} not page-aligned",
        start.0
    );
    assert!(
        size & 0xFFF == 0,
        "map_range: size {:#x} not page-aligned",
        size
    );

    let pages = size / PAGE_SIZE;
    for i in 0..pages {
        let virt = VirtAddr::new(start.0 + (i * PAGE_SIZE) as u32);
        map_alloc(virt, flags)?;
    }

    dbg_println!(
        "VMM: mapped range {:#x}..{:#x} ({} pages)",
        start.0,
        start.0 as usize + size,
        pages
    );

    Ok(pages)
}

// Map a contiguous virtual range to a specific contiguous physical region.
pub fn map_range_to(
    virt_start: VirtAddr,
    phys_start: PhysAddr,
    size: usize,
    flags: PageFlags,
) -> Result<usize, MapError> {
    assert!(
        virt_start.is_page_aligned(),
        "map_range_to: virt {:#x} not aligned",
        virt_start.0
    );
    assert!(
        phys_start.is_page_aligned(),
        "map_range_to: phys {:#x} not aligned",
        phys_start.0
    );
    assert!(
        size & 0xFFF == 0,
        "map_range_to: size {:#x} not aligned",
        size
    );

    let pages = size / PAGE_SIZE;
    for i in 0..pages {
        let virt = VirtAddr::new(virt_start.0 + (i * PAGE_SIZE) as u32);
        let phys = PhysAddr::new(phys_start.0 + (i * PAGE_SIZE) as u32);
        map_page(virt, phys, flags)?;
    }

    dbg_println!(
        "VMM: mapped range virt {:#x}..{:#x} -> phys {:#x}..{:#x} ({} pages)",
        virt_start.0,
        virt_start.0 as usize + size,
        phys_start.0,
        phys_start.0 as usize + size,
        pages
    );

    Ok(pages)
}

// Unmap a contiguous range and free their physical frames.
// Already-unmapped pages are silently skipped.
pub fn unmap_range(start: VirtAddr, size: usize) -> usize {
    assert!(
        start.is_page_aligned(),
        "unmap_range: start {:#x} not page-aligned",
        start.0
    );
    assert!(
        size & 0xFFF == 0,
        "unmap_range: size {:#x} not page-aligned",
        size
    );

    let pages = size / PAGE_SIZE;
    let mut unmapped = 0;
    for i in 0..pages {
        let virt = VirtAddr::new(start.0 + (i * PAGE_SIZE) as u32);
        if let Ok(phys) = unmap_page(virt) {
            free_frame(phys);
            unmapped += 1;
        }
    }

    dbg_println!(
        "VMM: unmapped range {:#x}..{:#x} ({}/{} pages freed)",
        start.0,
        start.0 as usize + size,
        unmapped,
        pages
    );

    unmapped
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn alloc_frame() -> Result<PhysFrame, MapError> {
    unsafe {
        FRAME_ALLOCATOR
            .as_mut()
            .ok_or(MapError::FrameAllocationFailed)?
            .allocate_frame()
            .map_err(|_| MapError::FrameAllocationFailed)
    }
}

fn free_frame(phys: PhysAddr) {
    unsafe {
        if let Some(alloc) = FRAME_ALLOCATOR.as_mut() {
            let frame = PhysFrame::containing_address(phys.0 as usize);
            let _ = alloc.deallocate_frame(frame);
        }
    }
}

// ---------------------------------------------------------------------------
// Self-test suite
// ---------------------------------------------------------------------------

pub fn test_virtual_memory() {
    m_println!("\n=== VMM Self-Test ===\n");

    test_recursive_mapping_reads();
    test_map_write_read_unmap();
    test_translate_accuracy();
    test_multi_page();
    test_already_mapped_error();
    test_map_range();
    m_println!("\n=== VMM Self-Test PASSED ===\n");
}

fn test_recursive_mapping_reads() {
    m_print!("[VMM test 1] Recursive mapping reads ... ");
    unsafe {
        let pde_768 = read_pde(768);
        assert!(pde_768.present(), "PDE[768] should be present");

        let pde_rec = read_pde(RECURSIVE_INDEX);
        assert!(pde_rec.present(), "PDE[1023] (recursive) should be present");

        let pd = super::PAGING.as_mut().unwrap();
        let pd_phys = pd.physical_address();
        assert_eq!(
            pde_rec.address(),
            pd_phys,
            "Recursive PDE should point to page directory"
        );
    }
    m_println!("OK");
}

fn test_map_write_read_unmap() {
    m_print!("[VMM test 2] Map -> write -> read -> unmap ... ");

    let test_virt = VirtAddr::new(0xD000_0000);
    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;

    let phys = map_alloc(test_virt, flags).expect("map_alloc failed");
    assert!(phys.is_page_aligned(), "Allocated phys not aligned");

    unsafe {
        let ptr = test_virt.0 as *mut u32;
        ptr.write_volatile(0xDEAD_BEEF);
        let readback = ptr.read_volatile();
        assert_eq!(readback, 0xDEAD_BEEF, "Write/read mismatch");
    }

    let returned_phys = unmap_page(test_virt).expect("unmap failed");
    assert_eq!(
        returned_phys, phys,
        "Unmap should return original phys addr"
    );
    assert!(
        !is_mapped(test_virt),
        "Page should not be mapped after unmap"
    );
    free_frame(phys);

    m_println!("OK");
}

fn test_translate_accuracy() {
    m_print!("[VMM test 3] translate() accuracy ... ");

    let test_virt = VirtAddr::new(0xD000_1000);
    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;

    let phys = map_alloc(test_virt, flags).expect("map_alloc failed");

    let translated = translate(test_virt).expect("translate returned None");
    assert_eq!(
        translated.0, phys.0,
        "translate mismatch: got {:#x}, expected {:#x}",
        translated.0, phys.0
    );

    let with_offset = translate(VirtAddr::new(0xD000_1ABC)).expect("translate+offset None");
    assert_eq!(
        with_offset.0,
        phys.0 | 0xABC,
        "translate should preserve page offset"
    );

    let _ = unmap_page(test_virt);
    free_frame(phys);

    m_println!("OK");
}

fn test_multi_page() {
    m_print!("[VMM test 4] Multi-page mapping ... ");

    let base: u32 = 0xD010_0000;
    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;
    let count: u32 = 4;
    let mut phys_addrs: [PhysAddr; 4] = [PhysAddr(0); 4];

    for i in 0..count {
        let virt = VirtAddr::new(base + i * PAGE_SIZE as u32);
        let phys = map_alloc(virt, flags).expect("map_alloc failed in multi-page");
        phys_addrs[i as usize] = phys;
        unsafe {
            (virt.0 as *mut u32).write_volatile(0xCAFE_0000 + i);
        }
    }

    for i in 0..count {
        let virt = VirtAddr::new(base + i * PAGE_SIZE as u32);
        unsafe {
            let val = (virt.0 as *mut u32).read_volatile();
            assert_eq!(
                val,
                0xCAFE_0000 + i,
                "Multi-page readback mismatch at page {}",
                i
            );
        }
    }

    for i in 0..count {
        let virt = VirtAddr::new(base + i * PAGE_SIZE as u32);
        let _ = unmap_page(virt);
        free_frame(phys_addrs[i as usize]);
    }

    m_println!("OK");
}

fn test_already_mapped_error() {
    m_print!("[VMM test 5] AlreadyMapped error ... ");

    let test_virt = VirtAddr::new(0xD020_0000);
    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;

    let phys = map_alloc(test_virt, flags).expect("first map failed");

    let dummy_phys = PhysAddr::new(0x0020_0000);
    match map_page(test_virt, dummy_phys, flags) {
        Err(MapError::AlreadyMapped) => { /* expected */ }
        Ok(_) => panic!("Should have returned AlreadyMapped"),
        Err(e) => panic!("Wrong error: {:?}", e),
    }

    let _ = unmap_page(test_virt);
    free_frame(phys);

    m_println!("OK");
}

fn test_map_range() {
    m_print!("[VMM test 6] map_range / unmap_range ... ");

    let base = VirtAddr::new(0xD200_0000);
    let size = PAGE_SIZE * 8;
    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;

    let mapped = map_range(base, size, flags).expect("map_range failed");
    assert_eq!(mapped, 8, "Should have mapped 8 pages");

    for i in 0..8u32 {
        let addr = (base.0 + i * PAGE_SIZE as u32) as *mut u32;
        unsafe {
            addr.write_volatile(0xBEEF_0000 + i);
        }
    }
    for i in 0..8u32 {
        let addr = (base.0 + i * PAGE_SIZE as u32) as *mut u32;
        unsafe {
            let val = addr.read_volatile();
            assert_eq!(
                val,
                0xBEEF_0000 + i,
                "map_range readback mismatch at page {}",
                i
            );
        }
    }

    for i in 0..8u32 {
        let virt = VirtAddr::new(base.0 + i * PAGE_SIZE as u32);
        assert!(is_mapped(virt), "Page {} should be mapped", i);
    }

    let freed = unmap_range(base, size);
    assert_eq!(freed, 8, "Should have unmapped 8 pages");

    for i in 0..8u32 {
        let virt = VirtAddr::new(base.0 + i * PAGE_SIZE as u32);
        assert!(
            !is_mapped(virt),
            "Page {} should not be mapped after unmap_range",
            i
        );
    }

    m_println!("OK");
}
