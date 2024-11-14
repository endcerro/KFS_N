use core::{fmt, ptr::NonNull};

use super::define::PAGE_TABLE_ENTRIES;

const PRESENT : u32 = 1 << 0;
const WRITABLE : u32 = 1 << 1;
const USER : u32 = 1 << 2;
const PWT : u32 = 1 << 3;
const CACHE_DISABLE : u32 = 1 << 4;
const ACCESSED : u32 = 1 << 5;
const DIRTY : u32 = 1 << 6;
const PAT : u32 = 1 << 7;
const GLOBAL : u32 = 1 << 8;

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: NonNull<[PageTableEntry; PAGE_TABLE_ENTRIES]>,
}
impl PageTable {
    pub fn new(address: *mut [u32; PAGE_TABLE_ENTRIES]) -> Self {
        unsafe {
            PageTable {
                entries: core::ptr::NonNull::new_unchecked(address as *mut [PageTableEntry; 1024])
            }
        }
    }

    // pub fn set_entry(&mut self, index: usize, address: usize, flags: PageFlags) {
    //     unsafe {
    //         self.entries.as_mut()[index] = (address as u32 & 0xFFFFF000) | flags.value();
    //     }
    // }
    pub fn get_entry(&mut self, index: usize) -> *mut PageTableEntry {
            self.entries.as_ptr().wrapping_add(index) as  *mut PageTableEntry
    }
}


#[repr(C, align(4096))]
pub struct PageTableEntry(pub u32);

impl PageTableEntry {
    #[inline] pub fn present(&self) -> bool {
        self.0 & PRESENT > 0
    }
    #[inline] pub fn writeable(&self) -> bool {
        self.0 & WRITABLE > 0
    }
    #[inline] pub fn user(&self) -> bool {
        self.0 & USER > 0
    }
    #[inline] pub fn pwt(&self) -> bool {
        self.0 & PWT > 0
    }
    #[inline] pub fn cache_disable(&self) -> bool {
        self.0 & CACHE_DISABLE > 0
    }
    #[inline] pub fn accessed(&self) -> bool {
        self.0 & ACCESSED > 0
    }
    #[inline] pub fn dirty(&self) -> bool {
        self.0 & DIRTY > 0
    }
    #[inline] pub fn pat(&self) -> bool {
        self.0 & PAT > 0
    }
    #[inline] pub fn global(&self) -> bool {
        self.0 & GLOBAL > 0
    }
    #[inline] pub fn address(&self) -> u32 {
        self.0 >> 12
    }
}

impl fmt::Display for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f, "Present {},writeable {},user {},pwt {},cache disable {},accessed {},dirty {},pat {},global {}, address {:x}",
         self.present(), self.writeable(), self.user(), self.pwt(), self.cache_disable(), self.accessed(), self.dirty(), self.pat(), self.global(), self.address())
    }
}