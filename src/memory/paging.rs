

use spin::Mutex;
use lazy_static::lazy_static;
pub const PAGESIZE: usize = 1024;

lazy_static! {
	pub static ref PAGEDIR : Mutex<[u32; PAGESIZE]> = Mutex::new([0; 1024]);
    pub static ref FPAGE : Mutex<[u32; PAGESIZE]> = Mutex::new([0; 1024]);
}



extern "C" 
{
    fn loadpagedirectory(a : *const u32);
    fn enablepaging();
}
pub fn test_paging()
{
    // let mut page_directory : [u32; PAGESIZE] = [0; PAGESIZE];
    // let mut first_page : [u32; PAGESIZE] = [0; PAGESIZE];

    // page_directory.eac
    for (_idx, obj) in PAGEDIR.lock().iter_mut().enumerate() {
        *obj = 0x00000002;
    }

    for (idx, obj) in FPAGE.lock().iter_mut().enumerate() {
        *obj = (idx as u32 * 0x1000 ) | 3;
    }
    PAGEDIR.lock()[0] = FPAGE.lock().as_ptr() as u32 | 3;

    unsafe {
        loadpagedirectory(PAGEDIR.lock().as_ptr() as *const u32);
        enablepaging();
    }
}