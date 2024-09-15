#![no_std]
#![feature(abi_x86_interrupt)]
#![no_main]
#[macro_use]
pub mod vga;
pub mod gdt;
pub mod keyboard;
//pub mod memory;
pub mod multiboot2;
pub mod utils;
pub mod interrupts;
pub mod serial;

use core::{panic::PanicInfo, ptr::addr_of};

use keyboard::{get_next_key_event, ControlKey, KeyCode};
use vga::WRITER;

// use keyboard::{KeyCode, KEYBOARD_BUFFER};

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


    loop {
        loop {
            if let Some(event) = get_next_key_event() 
            {
                if event.pressed == true
                {
                    // println!("{event}");
                    match event.code {
                        KeyCode::Control(ControlKey::Enter) => break,
                        KeyCode::Char(c) => print!("{c}"),
                        KeyCode::Control(ControlKey::Backspace) => WRITER.lock().delete_char(),
                        _ => ()
                    }

                }
        }
    }
    let len = keyboard::get_input_string();
    println!("\nFrom readline : {len}\n");

}



}

fn init() {
    serial::init();
    vga::clear_screen();
    vga::print_ft();
    gdt::init();
    interrupts::init();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}
