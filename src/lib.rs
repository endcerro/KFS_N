#![no_std]
#![feature(abi_x86_interrupt)]
#![no_main]
#[macro_use]
pub mod vga;
pub mod gdt;
pub mod pic;
pub mod keyboard;
//pub mod memory;
pub mod multiboot2;
pub mod utils;
pub mod idt;
pub mod handlers;
pub mod interrupts;
pub mod serial;


use core::{panic::PanicInfo, ptr::addr_of};

extern "C" {
    static _kernel_start : u8;
    static _kernel_end : u8;
}

#[no_mangle]
pub extern "C" fn rust_main(_multiboot_struct_ptr: *const multiboot2::MultibootInfoHeader) -> ! {
    init();
    // unsafe {
    let size = addr_of!(_kernel_end) as u32 - addr_of!(_kernel_start) as u32 ;
    //     // size = size /8;
    println!("The size of this kernel is {} kbytes", size / (1024));
    println!("The size of this kernel is {} mbytes", (size / (1024 * 1024)));
    serial_println!("Hello from serial port!");
    serial_println!("Kernel size: {} kbytes", size / 1024);
    
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
    serial::init();
    vga::clear_screen();
    vga::print_ft();
    gdt::init();
    idt::init();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}
