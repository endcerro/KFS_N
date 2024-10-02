// src/memory/paging.rs

use core::{arch::asm, ptr::addr_of};

pub const PAGE_SIZE: usize = 4096;
const PAGE_TABLE_ENTRIES: usize = 1024;
const PAGE_DIRECTORY_ENTRIES: usize = 1024;

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [u32; PAGE_TABLE_ENTRIES],
}

#[repr(C, align(4096))]
pub struct PageDirectory {
    entries: [u32; PAGE_DIRECTORY_ENTRIES],
}

pub struct Paging {
    page_directory: PageDirectory,
    page_tables: [PageTable; PAGE_DIRECTORY_ENTRIES],
}

impl Paging {
    pub const fn new() -> Self {
        const EMPTY_PAGE_TABLE: PageTable = PageTable { entries: [0; PAGE_TABLE_ENTRIES] };
        Paging {
            page_directory: PageDirectory { entries: [0; PAGE_DIRECTORY_ENTRIES] },
            page_tables: [EMPTY_PAGE_TABLE; PAGE_DIRECTORY_ENTRIES],
        }
    }

    pub fn init(&mut self) {
        // Identity map the first 4MB
        for i in 0..PAGE_TABLE_ENTRIES {
            self.page_tables[0].entries[i] = (i * PAGE_SIZE) as u32 | 0x3; // Present + Writable
        }

        // Set up the page directory
        self.page_directory.entries[0] = addr_of!(self.page_tables[0]) as u32 | 0x3; // Present + Writable

        // Map kernel to higher half (0xC0000000)
        // let kernel_page_index = 0xC0000000 / (PAGE_SIZE * PAGE_TABLE_ENTRIES);
        // self.page_directory.entries[kernel_page_index] = addr_of!(self.page_tables[0]) as u32 | 0x3;

        // Load page directory
        unsafe {
            asm!("mov cr3, {}", in(reg) addr_of!(self.page_directory) as u32);
        }
    }

    pub fn enable_paging() {
        unsafe {
            asm!(
                "mov eax, cr0",
                "or eax, 0x80000000",
                "mov cr0, eax"
            );
        }
    }
}

pub static mut PAGING: Paging = Paging::new();