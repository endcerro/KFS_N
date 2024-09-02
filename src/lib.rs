#![no_std]
#![no_main]
#[macro_use]

pub mod vga;
pub mod gdt;
pub mod memory;
pub mod multiboot2;
pub mod utils;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn rust_main(_multiboot_struct_ptr: *const multiboot2::MultibootInfoHeader) -> ! {
    init();
    // gdt::print();
    // utils::print_kernel_stack();
    // multiboot2::init_mem(_multiboot_struct_ptr);
    // memory::init_paging( multiboot2::MultibootInfo::new(_multiboot_struct_ptr).get_memory_info().unwrap());
    memory::init_paging();
    print!("OK {}", size_of::<usize>());
    print!("OK {}", size_of::<u32>());
    loop {}
}

fn init() {
    vga::clear_screen();
    gdt::init();
    vga::print_ft();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}
