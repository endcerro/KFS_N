#![no_std]
#![no_main]
#[macro_use]

pub mod vga;
pub mod gdt;
//pub mod memory;
pub mod multiboot2;
pub mod utils;
use core::panic::PanicInfo;

extern "C" {
    static _kernel_start : u32;
    static _kernel_end : u32;

}

#[no_mangle]
pub extern "C" fn rust_main(_multiboot_struct_ptr: *const multiboot2::MultibootInfoHeader) -> ! {
    init();
    // unsafe {
    //     let mut size =  _kernel_start - _kernel_end;
    //     // size = size /8;
    //     println!("The size of this kernel is {} kbytes", size / (1024) );
    //     println!("The size of this kernel is {} kbytes", size / (1024 * 1024) );
    //     // print!("The size of this kernel is {} mbytes", size / 1024 / 1024);
    // }
    // gdt::print();
    // utils::print_kernel_stack();
    // multiboot2::init_mem(_multiboot_struct_ptr);
    // memory::init_paging( multiboot2::MultibootInfo::new(_multiboot_struct_ptr).get_memory_info().unwrap());
    // memory::init_paging();
    print!("OK {}", size_of::<usize>());
    print!("OK {}", size_of::<u32>());
    loop {}
}

fn init() {
    gdt::init();
    vga::clear_screen();
    vga::print_ft();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}
