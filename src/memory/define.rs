extern "C" {
    pub static page_directory: [u32; 1024];
    // static page_table1: [u32; 1024];
    static _kernel_start : u8;
    static _kernel_end : u8;
}

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_TABLE_ENTRIES: usize = 1024;
pub const PAGE_DIRECTORY_ENTRIES: usize = 1024;
pub const KERNEL_OFFSET: usize = 0xC0000000; // Higher half kernel offset