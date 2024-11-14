#![no_std]
#![feature(abi_x86_interrupt)]
#![no_main]
#![allow(static_mut_refs)]
#[macro_use]
pub mod vga;
pub mod gdt;
pub mod keyboard;
pub mod memory;
pub mod multiboot2;
pub mod utils;
pub mod interrupts;
pub mod serial;
pub mod shell;
pub mod boot;

use core::panic::PanicInfo;

use vga::WRITER;
use shell::shell_loop;


// use keyboard::{KeyCode, KEYBOARD_BUFFER};

extern "C" {
    static _kernel_start : u8;
    static _kernel_end : u8;
}

#[no_mangle]
pub extern "C" fn rust_main(_multiboot_struct_ptr: *const multiboot2::MultibootInfoHeader)  {
    // vga::clear_screen();
    // vga::print_ft();
    // unsafe {
    //     utils::enable_interrupts(false);
    // }

    init();
    // let size = addr_of!(_kernel_end) as u32 - addr_of!(_kernel_start) as u32 ;
    // println!("The size of this kernel is {} kbytes", size / (1024));
    // println!("The size of this kernel is {} mbytes", (size / (1024 * 1024)));
    // serial_println!("Hello from serial port!");
    // serial_println!("Kernel size: {} kbytes", size / 1024);
    shell_loop();
    loop{}

}
fn init() {
    vga::clear_screen();
    vga::print_ft();
    println!("Init paging...");

    memory::init();
    println!("Paging ok");

    WRITER.lock().change_color(Some(vga::Color::White), Some(vga::Color::Black));
    WRITER.lock().cursor.enable_cursor(0, 15);
    // serial::init();

    // serial_print!("SAMPLE");
    println!("Init GDT...");
    gdt::init();
    println!("GDT OK ");

    interrupts::init();
    shell::init_shell();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}






