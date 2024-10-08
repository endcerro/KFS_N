use core::{arch::asm, ptr::{addr_of, addr_of_mut}};

use pageflags::PageFlags;
use paging::{*};


pub mod paging;
pub mod pageflags;


extern "C" {
    static page_directory: [u32; 1024];
}

pub struct Paging {
    page_directory: PageDirectory,
    page_tables: [PageTable; PAGE_DIRECTORY_ENTRIES],
}

extern "C" {
    static _kernel_start : u8;
    static _kernel_end : u8;
}


impl Paging {
    pub const fn new() -> Self {
        const EMPTY_PAGE_TABLE: PageTable = PageTable { entries: [0; PAGE_TABLE_ENTRIES] };
        Paging {
            page_directory: PageDirectory::default(),//= PageDirectory::default(),
            page_tables: [EMPTY_PAGE_TABLE; PAGE_DIRECTORY_ENTRIES],
        }
    }

    pub fn init(&mut self) {
        // Identity map the first 4MB
        // self.map_range(0, 0, 4 * 1024 * 1024, PageFlags::PRESENT | PageFlags::WRITABLE);
        crate::println!("Initializing paging...");

        let kernel_start = addr_of!(_kernel_start) as usize;
        let kernel_end = addr_of!(_kernel_end) as usize;
        let kernel_size = kernel_end - kernel_start;

        crate::println!("Kernel start: 0x{:x}, end: 0x{:x}, size: 0x{:x}, oldsize : 0x{:x} ", 
                        kernel_start, kernel_end, kernel_size, 4 * 1024 * 1024);

        self.map_range(0, 0, kernel_start, PageFlags::PRESENT | PageFlags::WRITABLE);
        self.map_range(kernel_start, kernel_start, kernel_size, PageFlags::PRESENT | PageFlags::WRITABLE);

        // Map kernel to higher half
        // let kernel_physical_start = 0x100000; // Assuming kernel starts at 1MB physical address
        // let kernel_size = 4 * 1024 * 1024; // Assuming kernel size is 4MB
        // self.map_range(KERNEL_OFFSET, kernel_physical_start, kernel_size, PageFlags::PRESENT | PageFlags::WRITABLE);

        // Set up the page directory
        for i in 0..PAGE_DIRECTORY_ENTRIES {
            self.page_directory.set_entry(i, &self.page_tables[i], PageFlags::PRESENT | PageFlags::WRITABLE);
        }

        // Load page directory
        unsafe {
            core::arch::asm!("mov cr3, {}", in(reg) &page_directory as *const _ as u32);
        }
    }

    fn map_range(&mut self, virtual_start: usize, physical_start: usize, size: usize, flags: PageFlags) {
        let start_page = virtual_start / PAGE_SIZE;
        let end_page = (virtual_start + size - 1) / PAGE_SIZE;


        for page in start_page..=end_page {
            let dir_index = page / PAGE_TABLE_ENTRIES;
            let table_index = page % PAGE_TABLE_ENTRIES;
            let physical_address = physical_start + (page - start_page) * PAGE_SIZE;

            self.page_tables[dir_index].set_entry(table_index, physical_address, flags);
        }
    }

    pub fn enable_paging(&self) {
        unsafe {
            asm!(
                "mov eax, cr0",
                "or eax, 0x80000000",
                "mov cr0, eax"
            );
        }
    }
}


pub fn init() {
    unsafe {
        PAGING.init();
        PAGING.enable_paging();
    }
    // colored_print!((Some(Color::Green), Some(Color::Black)), "\nPAGING OK");
}

pub static mut PAGING: Paging = Paging::new();