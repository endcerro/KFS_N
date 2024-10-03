// src/memory/paging.rs

use super::pageflags::PageFlags;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_TABLE_ENTRIES: usize = 1024;
pub const PAGE_DIRECTORY_ENTRIES: usize = 1024;
pub const KERNEL_OFFSET: usize = 0xC0000000; // Higher half kernel offset

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
    pub entries: [u32; PAGE_DIRECTORY_ENTRIES],
}
impl PageDirectory {
    pub fn new() -> Self {
        PageDirectory { entries: [0; PAGE_DIRECTORY_ENTRIES] }
    }

    pub fn set_entry(&mut self, index: usize, table: &PageTable, flags: PageFlags) {
        self.entries[index] = ((table as *const PageTable as u32) & 0xFFFFF000) | flags.value();
    }
}

