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
//     • Page directory accessible at   0xFFFFF000
//       (PDE index 1023 → PD, PTE index 1023 → PD[1023] → PD again)
//
//     • Page table N accessible at     0xFFC00000 + N * 0x1000
//       (PDE index 1023 → PD, PTE index N → PD[N] → page table N)
//
//   So to read/write PDE[i]:  *(0xFFFFF000 + i*4)
//   To read/write PT_N[j]:    *((0xFFC00000 + N*0x1000) + j*4)
//
// TLB discipline:
//   • After creating a new page table (setting a PDE), flush_tlb_all()
//     so the recursive mapping reflects the new PDE.
//   • After setting a PTE, invlpg the target virtual address.

use super::define::{KERNEL_OFFSET, PAGE_SIZE};
use super::pageflags::PageFlags;
use super::physical::{PhysFrame, FRAME_ALLOCATOR};

// ---------------------------------------------------------------------------
// Address wrapper types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub u32);

impl VirtAddr {
    #[inline] pub fn new(addr: u32) -> Self { VirtAddr(addr) }
    /// Bits [31:22] - selects one of 1024 page directory entries
    #[inline] pub fn pde_index(&self) -> usize { (self.0 >> 22) as usize }
    /// Bits [21:12] - selects one of 1024 page table entries
    #[inline] pub fn pte_index(&self) -> usize { ((self.0 >> 12) & 0x3FF) as usize }
    /// Bits [11:0] - byte offset within the 4 KB page
    #[inline] pub fn page_offset(&self) -> u32 { self.0 & 0xFFF }
    #[inline] pub fn is_page_aligned(&self) -> bool { self.0 & 0xFFF == 0 }
    /// Is this address in the kernel half (>= 0xC0000000)?
    #[inline] pub fn is_kernel(&self) -> bool { self.0 >= KERNEL_OFFSET as u32 }
}

impl PhysAddr {
    #[inline] pub fn new(addr: u32) -> Self { PhysAddr(addr) }
    #[inline] pub fn is_page_aligned(&self) -> bool { self.0 & 0xFFF == 0 }
}

// ---------------------------------------------------------------------------
// Recursive mapping constants
// ---------------------------------------------------------------------------

/// We sacrifice the last 4 MB of virtual space (0xFFC00000–0xFFFFFFFF)
/// for the recursive mapping.  This region is kernel-only.
const RECURSIVE_INDEX: usize = 1023;

/// Base virtual address where page table N is mapped:
///   page_table_virt(N) = PAGE_TABLES_VBASE + N * PAGE_SIZE
const PAGE_TABLES_VBASE: u32 = 0xFFC00000;

/// Virtual address of the page directory itself (= page_table_virt(1023))
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

// ---------------------------------------------------------------------------
// TLB helpers
// ---------------------------------------------------------------------------

/// Invalidate a single TLB entry.
/// NOTE: `invlpg` requires 486+; works on all modern x86 and QEMU i386.
#[inline]
pub fn flush_tlb_entry(virt: VirtAddr) {
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) virt.0, options(nostack, preserves_flags));
    }
}

/// Flush the entire TLB by reloading CR3.
#[inline]
pub fn flush_tlb_all() {
    unsafe {
        let cr3: u32;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack));
        core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack));
    }
}

// ---------------------------------------------------------------------------
// Raw PDE / PTE access via recursive mapping
// ---------------------------------------------------------------------------

/// Read PDE[index] through the recursive mapping.
#[inline]
unsafe fn read_pde(index: usize) -> u32 {
    let ptr = PAGE_DIR_VIRT as *const u32;
    ptr.add(index).read_volatile()
}

/// Write PDE[index] through the recursive mapping.
#[inline]
unsafe fn write_pde(index: usize, value: u32) {
    let ptr = PAGE_DIR_VIRT as *mut u32;
    ptr.add(index).write_volatile(value);
}

/// Read PTE[pte_index] inside page table `pde_index`.
#[inline]
unsafe fn read_pte(pde_index: usize, pte_index: usize) -> u32 {
    let pt_base = (PAGE_TABLES_VBASE + pde_index as u32 * PAGE_SIZE as u32) as *const u32;
    pt_base.add(pte_index).read_volatile()
}

/// Write PTE[pte_index] inside page table `pde_index`.
#[inline]
unsafe fn write_pte(pde_index: usize, pte_index: usize, value: u32) {
    let pt_base = (PAGE_TABLES_VBASE + pde_index as u32 * PAGE_SIZE as u32) as *mut u32;
    pt_base.add(pte_index).write_volatile(value);
}

// ---------------------------------------------------------------------------
// Initialisation
// ---------------------------------------------------------------------------

/// Set up recursive page directory mapping at PDE[1023].
///
/// Must be called once, early in boot, after `PAGING` is initialised.
/// After this, all map/unmap/translate operations use the recursive
/// mapping and do not need the PageDirectory wrapper.
pub fn init() {
    unsafe {
        let pd = super::PAGING.as_mut().expect("PAGING not initialised");
        let pd_phys = pd.physical_address();

        // PDE[1023] = page_directory_physical | PRESENT | WRITABLE
        pd.set_entry(
            RECURSIVE_INDEX,
            pd_phys,
            PageFlags::PRESENT | PageFlags::WRITABLE,
        );

        // Full TLB flush so the recursive mapping takes effect
        flush_tlb_all();
    }

    #[cfg(feature = "verbose")]
    println!("VMM: recursive mapping installed at PDE[{}]", RECURSIVE_INDEX);
}

// ---------------------------------------------------------------------------
// Core API
// ---------------------------------------------------------------------------

/// Map a single 4 KB virtual page to a physical frame.
///
/// If the page directory entry for this region has no page table yet,
/// a new frame is allocated and zeroed automatically.
///
/// `flags` are applied to the PTE.  The PDE is set to
/// PRESENT | WRITABLE (and USER if `flags` includes USER), which is the
/// standard "permissive PDE, restrictive PTE" policy.
pub fn map_page(virt: VirtAddr, phys: PhysAddr, flags: PageFlags) -> Result<(), MapError> {
    assert!(virt.is_page_aligned(), "virt addr {:#x} not page-aligned", virt.0);
    assert!(phys.is_page_aligned(), "phys addr {:#x} not page-aligned", phys.0);

    let pde_idx = virt.pde_index();
    let pte_idx = virt.pte_index();

    // The recursive slot is reserved - never map into it
    if pde_idx == RECURSIVE_INDEX {
        return Err(MapError::InvalidAddress);
    }

    unsafe {
        // ── Step 1: ensure a page table exists for this PDE ──────────
        let pde_val = read_pde(pde_idx);

        if pde_val & 0x1 == 0 {
            // No page table yet - allocate a physical frame for one
            let frame = alloc_frame()?;
            let pt_phys = frame.start_address() as u32;

            // PDE flags: permissive - the PTE is the real gatekeeper.
            // Set USER on the PDE if the caller wants user-accessible pages,
            // otherwise kernel-only.
            let mut pde_flags = PageFlags::PRESENT | PageFlags::WRITABLE;
            if flags.is_user() {
                pde_flags = pde_flags | PageFlags::USER;
            }

            write_pde(pde_idx, pt_phys | pde_flags.value());

            // Full TLB flush so the recursive mapping exposes the new PT.
            // (We're about to access it at PAGE_TABLES_VBASE + pde_idx * 0x1000.)
            flush_tlb_all();

            // Zero the fresh page table - all 1024 PTEs become non-present
            let pt_virt = (PAGE_TABLES_VBASE + pde_idx as u32 * PAGE_SIZE as u32) as *mut u8;
            core::ptr::write_bytes(pt_virt, 0, PAGE_SIZE);

            #[cfg(feature = "verbose")]
            println!(
                "VMM: allocated PT frame {:#x} for PDE[{}]",
                pt_phys, pde_idx
            );
        } else if flags.is_user() && (pde_val & PageFlags::USER.value()) == 0 {
            // Page table exists but PDE lacks USER - promote it.
            // This happens when a kernel PDE later gets a user mapping.
            write_pde(pde_idx, pde_val | PageFlags::USER.value());
            flush_tlb_all();
        }

        // ── Step 2: set the PTE ──────────────────────────────────────
        let pte_val = read_pte(pde_idx, pte_idx);
        if pte_val & 0x1 != 0 {
            return Err(MapError::AlreadyMapped);
        }

        write_pte(pde_idx, pte_idx, phys.0 | flags.value());

        // Invalidate only the target page
        flush_tlb_entry(virt);
    }

    #[cfg(feature = "verbose")]
    println!(
        "VMM: mapped virt {:#x} -> phys {:#x} (flags {:#x})",
        virt.0,
        phys.0,
        flags.value()
    );

    Ok(())
}

/// Convenience: allocate a physical frame *and* map it at `virt`.
///
/// Returns the physical address of the newly allocated frame.
pub fn map_alloc(virt: VirtAddr, flags: PageFlags) -> Result<PhysAddr, MapError> {
    let frame = alloc_frame()?;
    let phys = PhysAddr::new(frame.start_address() as u32);
    map_page(virt, phys, flags)?;
    Ok(phys)
}

/// Unmap a single 4 KB virtual page.
///
/// Returns the physical address the page was mapped to, so the caller
/// can free the frame if desired.
pub fn unmap_page(virt: VirtAddr) -> Result<PhysAddr, UnmapError> {
    let pde_idx = virt.pde_index();
    let pte_idx = virt.pte_index();

    unsafe {
        let pde_val = read_pde(pde_idx);
        if pde_val & 0x1 == 0 {
            return Err(UnmapError::PageTableNotPresent);
        }

        let pte_val = read_pte(pde_idx, pte_idx);
        if pte_val & 0x1 == 0 {
            return Err(UnmapError::NotMapped);
        }

        let phys = PhysAddr::new(pte_val & 0xFFFFF000);

        // Clear the PTE
        write_pte(pde_idx, pte_idx, 0);
        flush_tlb_entry(virt);

        #[cfg(feature = "verbose")]
        println!(
            "VMM: unmapped virt {:#x} (was phys {:#x})",
            virt.0, phys.0
        );

        Ok(phys)
    }
}

/// Translate a virtual address to its physical address by walking
/// PDE → PTE.  Returns `None` if any level is not present.
pub fn translate(virt: VirtAddr) -> Option<PhysAddr> {
    let pde_idx = virt.pde_index();
    let pte_idx = virt.pte_index();

    unsafe {
        let pde_val = read_pde(pde_idx);
        if pde_val & 0x1 == 0 {
            return None;
        }

        let pte_val = read_pte(pde_idx, pte_idx);
        if pte_val & 0x1 == 0 {
            return None;
        }

        Some(PhysAddr::new((pte_val & 0xFFFFF000) | virt.page_offset()))
    }
}

/// Quick predicate - is this virtual page currently mapped?
pub fn is_mapped(virt: VirtAddr) -> bool {
    translate(VirtAddr::new(virt.0 & !0xFFF)).is_some()
}


// ---------------------------------------------------------------------------
// Range operations
// ---------------------------------------------------------------------------

/// Map a contiguous range of virtual pages, allocating a fresh physical
/// frame for each one.  Both `start` and `size` must be page-aligned.
///
/// On success returns `Ok(number_of_pages_mapped)`.
/// On failure the pages that were already mapped are **not** rolled back
/// (simple policy - the caller should treat this as fatal or handle
/// cleanup itself).
pub fn map_range(start: VirtAddr, size: usize, flags: PageFlags) -> Result<usize, MapError> {
    assert!(start.is_page_aligned(), "map_range: start {:#x} not page-aligned", start.0);
    assert!(size & 0xFFF == 0, "map_range: size {:#x} not page-aligned", size);

    let pages = size / PAGE_SIZE;
    for i in 0..pages {
        let virt = VirtAddr::new(start.0 + (i * PAGE_SIZE) as u32);
        map_alloc(virt, flags)?;
    }

    #[cfg(feature = "verbose")]
    println!(
        "VMM: mapped range {:#x}..{:#x} ({} pages)",
        start.0, start.0 as usize + size, pages
    );

    Ok(pages)
}

/// Map a contiguous range of virtual pages to a specific contiguous
/// physical region.  Useful for identity-mapping hardware regions or
/// mapping a known physical block.  Both addresses and `size` must be
/// page-aligned.
pub fn map_range_to(
    virt_start: VirtAddr,
    phys_start: PhysAddr,
    size: usize,
    flags: PageFlags,
) -> Result<usize, MapError> {
    assert!(virt_start.is_page_aligned(), "map_range_to: virt {:#x} not aligned", virt_start.0);
    assert!(phys_start.is_page_aligned(), "map_range_to: phys {:#x} not aligned", phys_start.0);
    assert!(size & 0xFFF == 0, "map_range_to: size {:#x} not aligned", size);

    let pages = size / PAGE_SIZE;
    for i in 0..pages {
        let virt = VirtAddr::new(virt_start.0 + (i * PAGE_SIZE) as u32);
        let phys = PhysAddr::new(phys_start.0 + (i * PAGE_SIZE) as u32);
        map_page(virt, phys, flags)?;
    }

    #[cfg(feature = "verbose")]
    println!(
        "VMM: mapped range virt {:#x}..{:#x} -> phys {:#x}..{:#x} ({} pages)",
        virt_start.0, virt_start.0 as usize + size,
        phys_start.0, phys_start.0 as usize + size,
        pages
    );

    Ok(pages)
}

/// Unmap a contiguous range of virtual pages and free their physical
/// frames.  Both `start` and `size` must be page-aligned.
///
/// Pages that are already unmapped are silently skipped.
/// Returns the number of pages actually unmapped.
pub fn unmap_range(start: VirtAddr, size: usize) -> usize {
    assert!(start.is_page_aligned(), "unmap_range: start {:#x} not page-aligned", start.0);
    assert!(size & 0xFFF == 0, "unmap_range: size {:#x} not page-aligned", size);

    let pages = size / PAGE_SIZE;
    let mut unmapped = 0;
    for i in 0..pages {
        let virt = VirtAddr::new(start.0 + (i * PAGE_SIZE) as u32);
        if let Ok(phys) = unmap_page(virt) {
            free_frame(phys);
            unmapped += 1;
        }
        // Silently skip pages that aren't mapped
    }

    #[cfg(feature = "verbose")]
    println!(
        "VMM: unmapped range {:#x}..{:#x} ({}/{} pages freed)",
        start.0, start.0 as usize + size, unmapped, pages
    );

    unmapped
}




// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Thin wrapper around the global frame allocator.
fn alloc_frame() -> Result<PhysFrame, MapError> {
    unsafe {
        FRAME_ALLOCATOR
            .as_mut()
            .ok_or(MapError::FrameAllocationFailed)?
            .allocate_frame()
            .map_err(|_| MapError::FrameAllocationFailed)
    }
}

/// Free a physical frame back to the allocator.
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

/// Run after init() to verify the VMM is operational.
/// Tests recursive mapping reads, map/unmap/translate, multi-page mapping,
/// and automatic page-table creation.
pub fn test_virtual_memory() {
    println!("\n=== VMM Self-Test ===\n");

    test_recursive_mapping_reads();
    test_map_write_read_unmap();
    test_translate_accuracy();
    test_multi_page();
    test_already_mapped_error();
    test_map_range();
    println!("\n=== VMM Self-Test PASSED ===\n");
}

/// Test 1 - Verify the recursive mapping can read known PDEs.
fn test_recursive_mapping_reads() {
    print!("[VMM test 1] Recursive mapping reads ... ");
    unsafe {
        // PDE[768] was set by bootstrap.asm - it must be present
        let pde_768 = read_pde(768);
        assert!(pde_768 & 0x1 != 0, "PDE[768] should be present");

        // PDE[1023] is our recursive entry - must be present
        let pde_rec = read_pde(RECURSIVE_INDEX);
        assert!(pde_rec & 0x1 != 0, "PDE[1023] (recursive) should be present");

        // The recursive PDE should point to the page directory itself
        let pd = super::PAGING.as_mut().unwrap();
        let pd_phys = pd.physical_address();
        assert_eq!(
            pde_rec & 0xFFFFF000,
            pd_phys,
            "Recursive PDE should point to page directory"
        );
    }
    println!("OK");
}

/// Test 2 - Map a page, write a value, read it back, then unmap.
fn test_map_write_read_unmap() {
    print!("[VMM test 2] Map → write → read → unmap ... ");

    // 0xD0000000 is PDE[832], which should be unmapped
    let test_virt = VirtAddr::new(0xD000_0000);
    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;

    // Allocate + map
    let phys = map_alloc(test_virt, flags).expect("map_alloc failed");
    assert!(phys.is_page_aligned(), "Allocated phys not aligned");

    // Write a magic value through the virtual address
    unsafe {
        let ptr = test_virt.0 as *mut u32;
        ptr.write_volatile(0xDEAD_BEEF);
        let readback = ptr.read_volatile();
        assert_eq!(readback, 0xDEAD_BEEF, "Write/read mismatch");
    }

    // Unmap and verify
    let returned_phys = unmap_page(test_virt).expect("unmap failed");
    assert_eq!(returned_phys, phys, "Unmap should return original phys addr");
    assert!(!is_mapped(test_virt), "Page should not be mapped after unmap");

    // Free the frame
    free_frame(phys);

    println!("OK");
}

/// Test 3 - Verify translate() returns the correct physical address.
fn test_translate_accuracy() {
    print!("[VMM test 3] translate() accuracy ... ");

    let test_virt = VirtAddr::new(0xD000_1000); // same PDE[832], PTE[1]
    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;

    let phys = map_alloc(test_virt, flags).expect("map_alloc failed");

    // translate of the page-aligned address should give the frame start
    let translated = translate(test_virt).expect("translate returned None");
    assert_eq!(
        translated.0, phys.0,
        "translate mismatch: got {:#x}, expected {:#x}",
        translated.0, phys.0
    );

    // translate with an offset should preserve the offset
    let with_offset = translate(VirtAddr::new(0xD000_1ABC)).expect("translate+offset None");
    assert_eq!(
        with_offset.0,
        phys.0 | 0xABC,
        "translate should preserve page offset"
    );

    // Cleanup
    let _ = unmap_page(test_virt);
    free_frame(phys);

    println!("OK");
}

/// Test 4 - Map several consecutive pages, write distinct values, read back.
fn test_multi_page() {
    print!("[VMM test 4] Multi-page mapping ... ");

    let base: u32 = 0xD010_0000; // PDE[836], fresh page table needed
    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;
    let count: u32 = 4;

    let mut phys_addrs: [PhysAddr; 4] = [PhysAddr(0); 4];

    // Map 4 consecutive pages
    for i in 0..count {
        let virt = VirtAddr::new(base + i * PAGE_SIZE as u32);
        let phys = map_alloc(virt, flags).expect("map_alloc failed in multi-page");
        phys_addrs[i as usize] = phys;

        // Write page index as marker
        unsafe {
            (virt.0 as *mut u32).write_volatile(0xCAFE_0000 + i);
        }
    }

    // Read back and verify each page still has its marker
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

    // Cleanup
    for i in 0..count {
        let virt = VirtAddr::new(base + i * PAGE_SIZE as u32);
        let _ = unmap_page(virt);
        free_frame(phys_addrs[i as usize]);
    }

    println!("OK");
}

/// Test 5 - Mapping an already-mapped page should return AlreadyMapped.
fn test_already_mapped_error() {
    print!("[VMM test 5] AlreadyMapped error ... ");

    let test_virt = VirtAddr::new(0xD020_0000);
    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;

    let phys = map_alloc(test_virt, flags).expect("first map failed");

    // Second mapping at the same address should fail
    let dummy_phys = PhysAddr::new(0x0020_0000);
    match map_page(test_virt, dummy_phys, flags) {
        Err(MapError::AlreadyMapped) => { /* expected */ }
        Ok(_) => panic!("Should have returned AlreadyMapped"),
        Err(e) => panic!("Wrong error: {:?}", e),
    }

    // Cleanup
    let _ = unmap_page(test_virt);
    free_frame(phys);

    println!("OK");
}

/// Test 6 - map_range / unmap_range for contiguous multi-page regions.
fn test_map_range() {
    print!("[VMM test 6] map_range / unmap_range ... ");

    // Use a region that hasn't been touched by other tests (PDE[840])
    let base = VirtAddr::new(0xD200_0000);
    let size = PAGE_SIZE * 8; // 8 pages = 32 KB
    let flags = PageFlags::PRESENT | PageFlags::WRITABLE;

    // Map the range
    let mapped = map_range(base, size, flags).expect("map_range failed");
    assert_eq!(mapped, 8, "Should have mapped 8 pages");

    // Write a distinct marker to each page and read back
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
            assert_eq!(val, 0xBEEF_0000 + i, "map_range readback mismatch at page {}", i);
        }
    }

    // All pages should be mapped
    for i in 0..8u32 {
        let virt = VirtAddr::new(base.0 + i * PAGE_SIZE as u32);
        assert!(is_mapped(virt), "Page {} should be mapped", i);
    }

    // Unmap the range
    let freed = unmap_range(base, size);
    assert_eq!(freed, 8, "Should have unmapped 8 pages");

    // Verify none are mapped any more
    for i in 0..8u32 {
        let virt = VirtAddr::new(base.0 + i * PAGE_SIZE as u32);
        assert!(!is_mapped(virt), "Page {} should not be mapped after unmap_range", i);
    }

    println!("OK");
}