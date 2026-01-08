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

use vga::{Color, WRITER};
use shell::shell_loop;


extern "C" {
    static _kernel_start : u8;
    static _kernel_end : u8;
}

#[no_mangle]
pub extern "C" fn rust_main(multiboot_struct_ptr: *const multiboot2::MultibootInfoHeader)  {
    multiboot2::bind_header(multiboot_struct_ptr);
    init();
    unsafe {
        utils::enable_interrupts(false);
        core::arch::asm!("hlt");
    }
    shell_loop();
}

fn init() {
    vga::clear_screen();
    vga::print_ft();

    memory::init();
    // print!("Paging     ");
    // colored_print!((None, Some(Color::Green)), "OK\n");
    multiboot2::meminfo::print_meminfo();
    WRITER.lock().change_color(Some(vga::Color::White), Some(vga::Color::Black));
    WRITER.lock().cursor.enable_cursor(0, 15);
    // print!("Serial     ");
    serial::init();
    // colored_print!((None, Some(Color::Green)), "OK\n");
    serial_println!("Hello world");
    // print!("GDT        ");
    gdt::init();
    // colored_print!((None, Some(Color::Green)), "OK\n");

    // print!("Interrupts ");
    interrupts::init();
    // colored_print!((None, Some(Color::Green)), "OK\n");
    // print!("Shell      ");
    shell::init_shell();
    // colored_print!((None, Some(Color::Green)), "OK\n");


}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}






