use directory::PageDirectory;


pub mod pageflags;
mod pagetable;
pub mod define;


pub mod directory;
pub mod physical;



extern "C" {
    pub fn clear_page1();
}

pub fn init() {
    unsafe {
        #[cfg(feature = "verbose")]
        println!("Entry 0 : {}", *PAGING.get_entry(0).get_entry(0));
        #[cfg(feature = "verbose")]
        println!("Cleaning..");
        clear_page1();
        #[cfg(feature = "verbose")]
        println!("Entry 0 : {}", *PAGING.get_entry(0).get_entry(0));


    }
    // colored_print!((Some(Color::Green), Some(Color::Black)), "\nPAGING OK");
}

pub static mut PAGING: PageDirectory = PageDirectory::default();