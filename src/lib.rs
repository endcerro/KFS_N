#![no_std]
#![no_main]

// use multiboot2::{BootInformation, BootInformationHeader};

#[macro_use]
pub mod vga;
// pub mod interrupts;
// pub mod multiboot;
// pub mod gdt;
use core::panic::PanicInfo;

#[no_mangle]
// pub extern "C" fn rust_main(multiboot_struct_ptr: *const multiboot::MultibootInfoHeader, _multiboot_magic_nbr : usize  ) -> ! {
pub extern "C" fn rust_main(multiboot_struct_ptr: usize, _multiboot_magic_nbr : usize  ) -> ! {
    init();
    loop {}
}

fn init() {
    vga::clear_screen();
    vga::print_ft();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}