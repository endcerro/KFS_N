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
    init_physical_memory();
    unsafe {
        #[cfg(feature = "verbose")]
        println!("Entry 0 : {}", *PAGING.get_entry(0));

        #[cfg(feature = "verbose")]
        println!("Entry 0 : {}", *PAGING.get_entry(0));
        #[cfg(feature = "verbose")]
        println!("Cleaning..");
        clear_page1();
        #[cfg(feature = "verbose")]
        println!("Entry 0 : {}", *PAGING.get_entry(0));


    }
    // colored_print!((Some(Color::Green), Some(Color::Black)), "\nPAGING OK");
}

// src/memory/mod.rs - Updated init_physical_memory function
fn init_physical_memory() {
    // Get memory map from multiboot (now returns a slice reference)
    if let Some(memory_map) = crate::multiboot2::meminfo::get_memory_map() {
        #[cfg(feature = "verbose")]
        println!("Initializing physical memory allocator with {} memory regions...", memory_map.len());

        physical::init_frame_allocator(memory_map);

        #[cfg(feature = "verbose")]
        println!("Physical memory allocator initialized");
    } else {
        panic!("Failed to get memory map from multiboot!");
    }
}

pub static mut PAGING: PageDirectory = PageDirectory::default();