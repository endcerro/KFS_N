use lazy_static::lazy_static;
use spin::Mutex;

extern "C" {
	static mut _kernel_end: u32;
	static mut _kernel_start: u32;

}

const PAGE_SIZE: usize = 4096;
const PAGE_TABLE_ENTRIES: usize = 1024;
const PAGE_DIRECTORY_ENTRIES: usize = 1024;

#[repr(transparent)]
#[derive(Clone, Copy)]
struct PageTableEntry(u32);

impl PageTableEntry {
	//https://wiki.osdev.org/images/6/60/Page_table_entry.png
	const PRESENT: u32 = 1 << 0;
    const WRITABLE: u32 = 1 << 1;
    const USER_ACCESSIBLE: u32 = 1 << 2;
	const WRITE_THROUGH_CACHE: u32 = 1 << 3;
	const CACHE_DISABLE: u32 = 1 << 4;
	const ACCESSED: u32 = 1 << 5;
	const DIRTY: u32 = 1 << 6;
	const PAGE_ATTRIVUTE: u32 = 1 << 7;
	const GLOBAL: u32 = 1 << 8;

	fn new(page_frame: u32, flags: u32) -> PageTableEntry {
		assert_eq!(0, page_frame & 0xFFF);
		PageTableEntry(page_frame | flags)
	}
}
pub struct PageTable([PageTableEntry; 1024]);

lazy_static!{
    static ref PAGE_TAB: Mutex<PageTable> = Mutex::new(PageTable([PageTableEntry(0); PAGE_TABLE_ENTRIES]));
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct PageDirectoryEntry(u32);

#[repr(transparent)]
pub struct PageDirectory([PageDirectoryEntry; 1024]);

//static mut PAGE_DIR : PageDirectory = PageDirectory([PageDirectoryEntry(0); PAGE_DIRECTORY_ENTRIES]);

// static PAGE_DIRECTORY : PageDirectory = PageDirectory{
// 	0 : [0; PAGE_DIRECTORY_ENTRIES]
// };

pub fn init_paging()
{
	PAGE_TAB.lock().0[0] = PageTableEntry(1);

}