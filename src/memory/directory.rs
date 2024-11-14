use core::{fmt, ptr::NonNull};


// use crate::define;

const PRESENT :u32 = 1<<0;
const WRITABLE :u32 = 1<<1;
const USER :u32 = 1<<2;
const PWT: u32 = 1<<3;
const CACHE_DISABLE: u32 = 1 << 4;
const ACCESSED: u32 = 1 << 5;
const AVAILABLE: u32 = 1 << 6;
const PAGESIZE4MB: u32 = 1 << 7;

#[repr(C, align(4096))]
pub struct PageDirectory {
    pub entries: NonNull<[PageDirectoryEntry; super::define::PAGE_DIRECTORY_ENTRIES]>,
    // pub entries: [u32; PAGE_DIRECTORY_ENTRIES],
}
impl PageDirectory {
    pub fn new() -> Self {
        unsafe {
            PageDirectory {
                // entries: [0; PAGE_DIRECTORY_ENTRIES]
                entries: NonNull::new_unchecked((
                    &super::define::page_directory as *const [u32; 1024]) as *mut [PageDirectoryEntry; 1024])
            }
        }
    }
    pub const fn default() -> Self {
        unsafe {
            PageDirectory {
                // entries: [0; PAGE_DIRECTORY_ENTRIES]
                entries: NonNull::new_unchecked((
                    &super::define::page_directory as *const [u32; 1024]) as *mut [PageDirectoryEntry; 1024])
            }

        }
    }

    // pub fn set_entry(&mut self, index: usize, table: &PageTable, flags: PageFlags) {
    //     unsafe {
    //         self.entries.as_mut()[index] = ((table as *const PageTable as u32) & 0xFFFFF000) | flags.value();
    //     }
    // }

    pub fn get_entry(&mut self, index:usize) -> *mut PageDirectoryEntry {
        self.entries.as_ptr().wrapping_add(index) as *mut PageDirectoryEntry
    }
}

pub struct PageDirectoryEntry(u32);
impl PageDirectoryEntry {
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
    #[inline] pub fn available(&self) -> bool {
        self.0 & AVAILABLE > 0
    }
    #[inline] pub fn pagesize4mb(&self) -> bool {
        self.0 & PAGESIZE4MB > 0
    }
    #[inline] pub fn address(&self) -> u32 {
        self.0 >> 12
    }
}

impl fmt::Display for PageDirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Present {},writeable {},user {},pwt {},cache disable {},accessed {},available {},4mb {}, address {:x}",
        self.present(), self.writeable(), self.user(), self.pwt(), self.cache_disable(), self.accessed(), self.available(), self.pagesize4mb(), self.address())
    }
}