// memory/paging.rs - Unified page entry and page directory types
//
// On i386, page directory entries (PDEs) and page table entries (PTEs)
// are both 32-bit words with the same layout:
//
//   Bits [31:12]  20-bit physical page frame address
//   Bits [11:0]   flags (present, writable, user, etc.)
//
// The only difference is the *meaning* of two bits:
//   Bit 6:  "Available" in a PDE,  "Dirty" in a PTE
//   Bit 7:  "Page Size (4MB)" in a PDE,  "PAT" in a PTE
//
// These are the same hardware bits - what changes is interpretation.
// So we use a single `PageEntry` type for both, with dual-named
// accessors for the context-dependent bits.
//
// Similarly, the "page directory" and "page table" are both arrays of
// 1024 PageEntry values.  The only wrapper we keep is `PageDirectory`,
// which exists solely for the early boot init path (before the VMM's
// recursive mapping is live).  After vmm::init(), all PDE/PTE access
// goes through the recursive mapping using raw PageEntry pointers.

use super::define::{KERNEL_OFFSET, PAGE_DIRECTORY_ENTRIES};
use super::pageflags::PageFlags;
use core::fmt;
use core::ptr::NonNull;

// ---------------------------------------------------------------------------
// PageEntry - a single 4-byte page directory or page table entry
// ---------------------------------------------------------------------------
//
// #[repr(C)] with no alignment override: size = 4, align = 4.
// This is critical - pointer arithmetic (e.g. ptr.add(index)) must
// stride by 4 bytes, not 4096.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PageEntry(u32);

impl PageEntry {
    // Create an entry pointing to `phys_addr` with the given flags.
    // Panics (debug) if `phys_addr` is not 4 KB aligned.
    pub fn new(phys_addr: u32, flags: PageFlags) -> Self {
        debug_assert!(
            phys_addr & 0xFFF == 0,
            "PageEntry address {:#x} must be 4KB aligned",
            phys_addr
        );
        PageEntry(flags.to_entry(phys_addr))
    }

    // An empty entry (non-present, value 0).
    pub const fn empty() -> Self {
        PageEntry(0)
    }

    // Overwrite this entry with a new physical address and flags.
    pub fn set(&mut self, phys_addr: u32, flags: PageFlags) {
        debug_assert!(
            phys_addr & 0xFFF == 0,
            "PageEntry address {:#x} must be 4KB aligned",
            phys_addr
        );
        self.0 = flags.to_entry(phys_addr);
    }

    // Clear this entry (mark as not present).
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    // -- Raw access ----------------------------------------------------------

    // Raw u32 value as the hardware sees it.
    #[inline]
    pub fn value(&self) -> u32 {
        self.0
    }

    // Extract the flags portion (low 12 bits) as PageFlags.
    #[inline]
    pub fn flags(&self) -> PageFlags {
        PageFlags::flags_of(self.0)
    }

    // Extract the 4 KB-aligned physical address (high 20 bits).
    #[inline]
    pub fn address(&self) -> u32 {
        PageFlags::addr_of(self.0)
    }

    // Page frame number (address >> 12).
    #[inline]
    pub fn page_frame_number(&self) -> u32 {
        self.0 >> 12
    }

    // -- Common flag queries (identical for PDE and PTE) ---------------------

    #[inline]
    pub fn present(&self) -> bool {
        self.flags().is_present()
    }
    #[inline]
    pub fn writeable(&self) -> bool {
        self.flags().is_writable()
    }
    #[inline]
    pub fn user(&self) -> bool {
        self.flags().is_user()
    }
    #[inline]
    pub fn write_through(&self) -> bool {
        self.flags().contains(PageFlags::WRITE_THROUGH)
    }
    #[inline]
    pub fn cache_disable(&self) -> bool {
        self.flags().contains(PageFlags::CACHE_DISABLE)
    }
    #[inline]
    pub fn accessed(&self) -> bool {
        self.flags().contains(PageFlags::ACCESSED)
    }
    #[inline]
    pub fn global(&self) -> bool {
        self.flags().contains(PageFlags::GLOBAL)
    }

    // -- Context-dependent bits (dual names) ---------------------------------
    //
    // Bit 6: "Dirty" when this is a PTE, "Available" when a PDE.
    //         Hardware only sets Dirty on PTEs.
    // Bit 7: "PAT" when this is a PTE, "Page Size (4MB)" when a PDE.
    //
    // We provide both names and let the caller pick the right one
    // based on whether they're looking at a PDE or PTE.

    // Bit 6 as a PTE field (dirty - set by hardware on write).
    #[inline]
    pub fn dirty(&self) -> bool {
        self.flags().contains(PageFlags::DIRTY)
    }

    // Bit 7 as a PDE field (4 MB page size / PSE).
    #[inline]
    pub fn page_size_4mb(&self) -> bool {
        self.flags().contains(PageFlags::HUGE_PAGE)
    }

    // Bit 7 as a PTE field (Page Attribute Table).
    #[inline]
    pub fn pat(&self) -> bool {
        self.flags().contains(PageFlags::HUGE_PAGE)
    }
}

impl fmt::Display for PageEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.present() {
            return write!(f, "[not present]");
        }
        write!(
            f,
            "phys {:#010x} {} accessed={} dirty={}",
            self.address(),
            self.flags(),
            self.accessed(),
            self.dirty(),
        )
    }
}

impl fmt::Debug for PageEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PageEntry({:#010x})", self.0)
    }
}

// ---------------------------------------------------------------------------
// PageDirectory - wrapper around the boot page directory
// ---------------------------------------------------------------------------
//
// This exists for the early init path: memory::init() creates it to
// point at the assembly-defined `page_directory` symbol, then uses it
// to install the recursive mapping at PDE[1023].  After vmm::init(),
// all access goes through the recursive mapping and this wrapper is
// no longer needed for runtime operations.
//
// We don't have a separate `PageTable` wrapper because:
//   1. The VMM accesses page tables through the recursive mapping.
//   2. A page table is structurally identical to a page directory:
//      1024 × PageEntry, 4 KB aligned.  If you ever need one, just
//      use a `[PageEntry; 1024]` or access it through a raw pointer.

pub struct PageDirectory {
    entries: NonNull<[PageEntry; PAGE_DIRECTORY_ENTRIES]>,
}

impl PageDirectory {
    // Wrap the assembly-defined `page_directory` symbol.
    pub fn new() -> Self {
        unsafe {
            PageDirectory {
                entries: NonNull::new_unchecked(
                    (&super::define::page_directory as *const [u32; 1024])
                        as *mut [PageEntry; 1024],
                ),
            }
        }
    }

    // Const version for static initialisation.
    pub const fn default() -> Self {
        unsafe {
            PageDirectory {
                entries: NonNull::new_unchecked(
                    (&super::define::page_directory as *const [u32; 1024])
                        as *mut [PageEntry; 1024],
                ),
            }
        }
    }

    // Set PDE[index] to point to `phys_addr` with `flags`.
    pub fn set_entry(&mut self, index: usize, phys_addr: u32, flags: PageFlags) {
        assert!(index < PAGE_DIRECTORY_ENTRIES, "PDE index out of bounds");
        assert!(
            phys_addr & 0xFFF == 0,
            "Page table address must be 4KB aligned"
        );

        unsafe {
            (*self.get_entry(index)).set(phys_addr, flags);
        }

        #[cfg(feature = "verbose")]
        println!(
            "PDE[{}] set to phys {:#x} flags {}",
            index, phys_addr, flags
        );
    }

    // Clear PDE[index] (mark as not present).
    pub fn clear_entry(&mut self, index: usize) {
        assert!(index < PAGE_DIRECTORY_ENTRIES, "PDE index out of bounds");

        unsafe {
            (*self.get_entry(index)).clear();
        }

        #[cfg(feature = "verbose")]
        println!("PDE[{}] cleared", index);
    }

    // Get a raw pointer to PDE[index]
    pub fn get_entry(&mut self, index: usize) -> *mut PageEntry {
        unsafe {
            let base = self.entries.as_ptr() as *mut PageEntry;
            base.add(index)
        }
    }

    // Physical address of this page directory
    // Assumes the directory lives in the higher half
    pub fn physical_address(&self) -> u32 {
        (self.entries.as_ptr() as u32).wrapping_sub(KERNEL_OFFSET as u32)
    }
}
