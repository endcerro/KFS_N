// src/memory/paging.rs
use core::ptr::NonNull;
use super::pageflags::PageFlags;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_TABLE_ENTRIES: usize = 1024;
pub const PAGE_DIRECTORY_ENTRIES: usize = 1024;
pub const KERNEL_OFFSET: usize = 0xC0000000; // Higher half kernel offset

extern "C" {
    static page_directory: [u32; 1024];
    static identity_page_table: [u32; 1024];
    static higher_half_page_table: [u32; 1024];
}

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u32; PAGE_TABLE_ENTRIES],
}
impl PageTable {
    pub fn new() -> Self {
        PageTable { entries: [0; PAGE_TABLE_ENTRIES] }
    }

    pub fn set_entry(&mut self, index: usize, address: usize, flags: PageFlags) {
        self.entries[index] = (address as u32 & 0xFFFFF000) | flags.value();
    }
}

#[repr(C, align(4096))]
pub struct PageDirectory {
    pub entries: NonNull<[u32; 1024]>,
    // pub entries: [u32; PAGE_DIRECTORY_ENTRIES],
}
impl PageDirectory {
    pub fn new() -> Self {
        unsafe {
            PageDirectory { 
                // entries: [0; PAGE_DIRECTORY_ENTRIES]
                entries: NonNull::new_unchecked((
                    &page_directory as *const [u32; 1024]) as *mut [u32; 1024])
            }
        }
    }
    pub const fn default() -> Self {
        unsafe {
            PageDirectory { 
                // entries: [0; PAGE_DIRECTORY_ENTRIES]
                entries: NonNull::new_unchecked((
                    &page_directory as *const [u32; 1024]) as *mut [u32; 1024])
            }

        }
    }

    pub fn set_entry(&mut self, index: usize, table: &PageTable, flags: PageFlags) {
        unsafe {
            self.entries.as_mut()[index] = ((table as *const PageTable as u32) & 0xFFFFF000) | flags.value();
        }
    }
}

