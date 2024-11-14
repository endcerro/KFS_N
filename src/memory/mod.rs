use directory::PageDirectory;


pub mod pageflags;
mod pagetable;
pub mod define;


pub mod directory;



extern "C" {
    pub fn clear_page1();
}

pub fn init() {
    unsafe {

        println!("Entry 0 : {}", *PAGING.get_entry(0).get_entry(0));
        println!("Cleaning..");
        clear_page1();
        println!("Entry 0 : {}", *PAGING.get_entry(0).get_entry(0));


    }
    // colored_print!((Some(Color::Green), Some(Color::Black)), "\nPAGING OK");
}

pub static mut PAGING: PageDirectory = PageDirectory::default();