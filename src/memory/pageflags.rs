// memory/pageflags.rs - Single source of truth for x86 page entry flags
//
// Both PageDirectoryEntry and PageTableEntry use these flags.
// No other file should define its own PRESENT / WRITABLE / etc. constants.

use core::fmt;
use core::ops::{BitAnd, BitOr, Not};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageFlags(u32);

impl PageFlags {
    pub const NONE: PageFlags = PageFlags(0);
    pub const PRESENT: PageFlags = PageFlags(1 << 0);
    pub const WRITABLE: PageFlags = PageFlags(1 << 1);
    pub const USER: PageFlags = PageFlags(1 << 2);
    pub const WRITE_THROUGH: PageFlags = PageFlags(1 << 3);
    pub const CACHE_DISABLE: PageFlags = PageFlags(1 << 4);
    pub const ACCESSED: PageFlags = PageFlags(1 << 5);
    pub const DIRTY: PageFlags = PageFlags(1 << 6);
    pub const HUGE_PAGE: PageFlags = PageFlags(1 << 7); // PSE in PDE, PAT in PTE
    pub const GLOBAL: PageFlags = PageFlags(1 << 8);

    // Mask that covers all 12 flag bits (bits 0-11).
    const FLAGS_MASK: u32 = 0xFFF;
    // Mask for the 20-bit physical page frame address (bits 12-31).
    pub const ADDR_MASK: u32 = 0xFFFFF000;

    // -- Constructors --------------------------------------------------------

    // Build PageFlags from a raw u32 (e.g. read from a hardware entry).
    // Only the low 12 bits are kept.
    #[inline]
    pub const fn from_raw(val: u32) -> Self {
        PageFlags(val & Self::FLAGS_MASK)
    }

    // -- Accessors -----------------------------------------------------------

    // Raw u32 value (only the low 12 bits are meaningful).
    #[inline]
    pub const fn value(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn is_present(self) -> bool {
        self.0 & Self::PRESENT.0 != 0
    }
    #[inline]
    pub const fn is_writable(self) -> bool {
        self.0 & Self::WRITABLE.0 != 0
    }
    #[inline]
    pub const fn is_user(self) -> bool {
        self.0 & Self::USER.0 != 0
    }

    // Check whether `self` contains all the bits in `other`.
    #[inline]
    pub const fn contains(self, other: PageFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    // -- Helpers for entry construction / decomposition ----------------------

    // Combine a page-aligned physical address with flags into the raw u32
    // that the hardware expects in a PDE or PTE.
    //
    // Panics (debug) if `phys_addr` is not 4 KB aligned.
    #[inline]
    pub fn to_entry(self, phys_addr: u32) -> u32 {
        debug_assert!(
            phys_addr & !Self::ADDR_MASK == 0,
            "address {:#x} is not 4 KB aligned",
            phys_addr
        );
        (phys_addr & Self::ADDR_MASK) | (self.0 & Self::FLAGS_MASK)
    }

    // Extract the flags portion from a raw PDE / PTE value.
    #[inline]
    pub const fn flags_of(raw: u32) -> PageFlags {
        PageFlags(raw & Self::FLAGS_MASK)
    }

    // Extract the 4 KB-aligned physical address from a raw PDE / PTE value.
    #[inline]
    pub const fn addr_of(raw: u32) -> u32 {
        raw & Self::ADDR_MASK
    }
}

// -- Bitwise operators -------------------------------------------------------

impl BitOr for PageFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        PageFlags(self.0 | rhs.0)
    }
}

impl BitAnd for PageFlags {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        PageFlags(self.0 & rhs.0)
    }
}

impl Not for PageFlags {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        PageFlags(!self.0)
    }
}

// -- Display / Debug ---------------------------------------------------------

impl fmt::Debug for PageFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PageFlags({:#05x})", self.0)
    }
}

impl fmt::Display for PageFlags {
    // Human-readable flag summary, e.g. "PWK" or "PRU"
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            if self.is_present() { 'P' } else { '-' },
            if self.is_writable() { 'W' } else { 'R' }, // W = writable, R = read-only
            if self.is_user() { 'U' } else { 'K' },     // U = user, K = kernel
        )
    }
}
