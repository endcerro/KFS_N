use core::ptr::NonNull;

use super::{pageflags::PageFlags, pagetable::PageTable};

// use crate::define;

#[repr(C, align(4096))]
pub struct PageDirectory {
    pub entries: NonNull<[u32; super::define::PAGE_DIRECTORY_ENTRIES]>,
    // pub entries: [u32; PAGE_DIRECTORY_ENTRIES],
}
impl PageDirectory {
    pub fn new() -> Self {
        unsafe {
            PageDirectory {
                // entries: [0; PAGE_DIRECTORY_ENTRIES]
                entries: NonNull::new_unchecked((
                    &super::define::page_directory as *const [u32; 1024]) as *mut [u32; 1024])
            }
        }
    }
    pub const fn default() -> Self {
        unsafe {
            PageDirectory {
                // entries: [0; PAGE_DIRECTORY_ENTRIES]
                entries: NonNull::new_unchecked((
                    &super::define::page_directory as *const [u32; 1024]) as *mut [u32; 1024])
            }

        }
    }



    pub fn set_entry(&mut self, index: usize, table: &PageTable, flags: PageFlags) {
        unsafe {
            self.entries.as_mut()[index] = ((table as *const PageTable as u32) & 0xFFFFF000) | flags.value();
        }
    }

    pub fn get_entry(&mut self, index:usize) -> PageTable {
        PageTable::new(self.entries.as_ptr().wrapping_add(index))
    }
}

