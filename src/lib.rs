#![no_std]
#![feature(abi_x86_interrupt)]
#![no_main]
#![allow(static_mut_refs)]
#[macro_use]
pub mod vga;
pub mod gdt;
pub mod keyboard;
pub mod memory;
//pub mod memory;
pub mod multiboot2;
pub mod utils;
pub mod interrupts;
pub mod serial;
pub mod shell;

use core::panic::PanicInfo;

use memory::paging::{Paging, PAGING};
use vga::WRITER;
use shell::shell_loop;


// use keyboard::{KeyCode, KEYBOARD_BUFFER};

extern "C" {
    static _kernel_start : u8;
    static _kernel_end : u8;
}

#[no_mangle]
pub extern "C" fn rust_main(_multiboot_struct_ptr: *const multiboot2::MultibootInfoHeader)  {
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
    WRITER.lock().change_color(Some(vga::Color::White), Some(vga::Color::Black));
    WRITER.lock().cursor.enable_cursor(0, 15);
    serial::init();
    vga::clear_screen();
    vga::print_ft();

    // unsafe { PAGING.init();
    // Paging::enable_paging();
//  };

    gdt::init();
    interrupts::init();;
    shell::init_shell();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}






