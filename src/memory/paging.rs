// use lazy_static::lazy_static;
// use spin::Mutex;
// pub const PAGESIZE: u32 = 1024;

// const PRESENT: u32 = 1 << 0;
// const READ_WRITE: u32 = 1 << 1;
// const READ_ONLY: u32 = 0 << 1;
// const USER_PAGE: u32 = 1 << 2;
// // const USER_PAGE: u32 = 1 << 2;
// const CACHE: u32 = 1 << 2;

// struct PageDirectoryEntry(u32);
// struct PageTableEntry(u32);

// //This should be enought to map 4GB
// struct PagingDirectory([PageDirectoryEntry; 1024]); //Each entry of this point to one below
// struct PageTable([PageTableEntry; 1024]); //And these point to physical frames

// impl PageDirectoryEntry {}
// lazy_static! {
//     pub static ref PAGEDIR: Mutex<[u32; PAGESIZE]> = Mutex::new([0; 1024]);
//     pub static ref FPAGE: Mutex<[u32; PAGESIZE]> = Mutex::new([0; 1024]);
// }

// extern "C" {
//     fn loadpagedirectory(a: *const u32);
//     fn enablepaging();
// }
// pub fn test_paging() {}
