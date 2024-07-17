#![no_std]
#![no_main]
#[macro_use]
pub mod vga;
pub mod gdt;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn rust_main(_multiboot_struct_ptr: usize, _multiboot_magic_nbr : usize) -> ! {
    init();
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