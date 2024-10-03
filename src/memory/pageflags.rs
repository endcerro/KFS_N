use core::ops::{BitOr, BitAnd, Not};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageFlags(u32);

impl PageFlags {
    pub const PRESENT: PageFlags = PageFlags(1 << 0);
    pub const WRITABLE: PageFlags = PageFlags(1 << 1);
    pub const USER: PageFlags = PageFlags(1 << 2);
    pub const WRITE_THROUGH: PageFlags = PageFlags(1 << 3);
    pub const CACHE_DISABLE: PageFlags = PageFlags(1 << 4);
    pub const ACCESSED: PageFlags = PageFlags(1 << 5);
    pub const DIRTY: PageFlags = PageFlags(1 << 6);
    pub const HUGE_PAGE: PageFlags = PageFlags(1 << 7);
    pub const GLOBAL: PageFlags = PageFlags(1 << 8);

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl BitOr for PageFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        PageFlags(self.0 | rhs.0)
    }
}

impl BitAnd for PageFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        PageFlags(self.0 & rhs.0)
    }
}

impl Not for PageFlags {
    type Output = Self;
    fn not(self) -> Self {
        PageFlags(!self.0)
    }
}