use physical::FRAME_ALLOCATOR;

use crate::{multiboot2::{meminfo::{self, MemoryInfoEntry}, MultibootInfo, MultibootInfoHeader}, serial, serial_println};

pub mod physical;



pub fn init(ptr : *mut MultibootInfoHeader)
{
    serial_println!("Let's find out how much memory there is :");

    
    let mut multiboot_info;
    // let mem_option;
    unsafe {
        multiboot_info = MultibootInfo::new(ptr);
        //mem_option = multiboot_info.get_memory_info().unwrap();
        FRAME_ALLOCATOR.init(&mut multiboot_info);
        FRAME_ALLOCATOR.print_bitmap_state();
    }

    // let mem = calculate_available_memory_bytes(&mut multiboot_info);
 

    // let mut mem_iterator = mem_option.entry;

    // let mut idx = 0;
    // loop {

    //     let entry = mem_iterator.next();
    //     match entry {
    //         None => break,
    //         Some(e) => unsafe {
    //             serial_println!("mem{1} {0}", *e, idx);
    //         } 
    //     }

    //     idx += 1;
    // }  


}



pub fn calculate_available_memory_bytes(multiboot_info: &mut MultibootInfo) -> u64 {
    let mut total_available = 0;
    
    if let Some(mem_info) = multiboot_info.get_memory_info() {
        let mut mem_iterator = mem_info.entry;
        
        loop {
            match mem_iterator.next() {
                None => break,
                Some(entry) => unsafe {
                    let entry: &MemoryInfoEntry = &*entry;
                    if entry.typee == 1 {  // Assuming type 1 is available memory
                        total_available += entry.length;
                    }
                }
            }
        }
    }
    serial_println!("There is {}b", total_available);
    serial_println!("There is {}kb", total_available / 1024);
    serial_println!("There is {}mb", total_available / 1024 /1024);
    total_available
}