use core::{fmt, ptr::NonNull};

use super::{define::PAGE_TABLE_ENTRIES, pageflags::PageFlags};

const PRESENT :u32 = 1<<0;
const WRITABLE :u32 = 1<<1;
const USER :u32 = 1<<2;
const PWT: u32 = 1<<3;
const CACHE_DISABLE: u32 = 1 << 4;
const ACCESSED: u32 = 1 << 5;
const AVAILABLE: u32 = 1 << 6;
const PAGESIZE4MB: u32 = 1 << 7;

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: NonNull<[u32; PAGE_TABLE_ENTRIES]>,
}
impl PageTable {
    pub fn new(address: *mut [u32; PAGE_TABLE_ENTRIES]) -> Self {
        unsafe {
            PageTable {
                entries: core::ptr::NonNull::new_unchecked(address) 
            }
        }
    }

    pub fn set_entry(&mut self, index: usize, address: usize, flags: PageFlags) {
        unsafe {
            self.entries.as_mut()[index] = (address as u32 & 0xFFFFF000) | flags.value();
        }
    }
    pub fn get_entry(&mut self, index: usize) -> *mut PageTableEntry {
            self.entries.as_ptr().wrapping_add(index) as  *mut PageTableEntry
    }
}


#[repr(C, align(4096))]
pub struct PageTableEntry(pub u32);

impl PageTableEntry {
    #[inline]
    pub fn present(&self) -> bool {
        self.0 & PRESENT > 0
    }
    #[inline]
    pub fn writeable(&self) -> bool {
        self.0 & WRITABLE > 0
    }
    #[inline]
    pub fn user(&self) -> bool {
        self.0 & USER > 0
    }
    #[inline]
    pub fn pwt(&self) -> bool {
        self.0 & PWT > 0
    }
    #[inline]
    pub fn cache_disable(&self) -> bool {
        self.0 & CACHE_DISABLE > 0
    }
    #[inline]
    pub fn accessed(&self) -> bool {
        self.0 & ACCESSED > 0
    }
    #[inline]
    pub fn available(&self) -> bool {
        self.0 & AVAILABLE > 0
    }
    #[inline]
    pub fn pagesize4mb(&self) -> bool {
        self.0 & PAGESIZE4MB > 0
    }
    #[inline]
    pub fn address(&self) -> u32 {
        self.0 >> 12
    }
}

impl fmt::Display for PageTableEntry { /*TODO Display access and flag with more granularity */
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.present() {
            write!(f, "Page is not present")
        }
        else {
            write!(f, "Page is present")
        }
        // write!(f, "Base {:x}, limit {:x}, flags {:x}, access {:x}", base, limit, flags, access)
    }
}