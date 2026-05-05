#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![no_main]
#![allow(static_mut_refs)]

extern crate alloc;
#[macro_use]
pub mod vga;
pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod memory;
pub mod multiboot2;
pub mod panic;
#[macro_use]
pub mod serial;
pub mod shell;
pub mod signals;
pub mod timer;
pub mod utils;

use core::panic::PanicInfo;

use shell::shell_loop;
use vga::WRITER;

use crate::vga::Color;
// ---------------------------------------------------------------------------
// Global allocator registration
// ---------------------------------------------------------------------------
//
// This tells the Rust alloc crate to route all heap allocations
// (Box::new, Vec::push, String::from, etc.) through our kernel heap.
// The KernelAllocator is a zero-sized struct - all state lives in
// heap.rs statics.
#[global_allocator]
static ALLOCATOR: memory::allocator::KernelAllocator = memory::allocator::KernelAllocator;

extern "C" {
    static _kernel_start: u8;
    static _kernel_end: u8;
}

#[no_mangle]
pub extern "C" fn rust_main() {
    init();
    unsafe {
        utils::enable_interrupts(true);
    }
    shell_loop();
}

fn init() {
    print!("Serial     ");
    serial::init();
    colored_print!((None, Some(Color::Green)), "OK\n");
    serial_println!(
        "==================================KERNEL START=================================="
    );
    vga::clear_screen();
    vga::print_ft();

    print!("MBOOT2     ");
    multiboot2::init();
    colored_print!((None, Some(Color::Green)), "OK\n");

    print!("Memory     ");
    memory::init();
    colored_print!((None, Some(Color::Green)), "OK\n");
    // memory::diagnose_page_directory();
    // memory::test_paging_infrastructure();
    // WRITER.lock().change_color(Some(vga::Color::White), Some(vga::Color::Black));
    WRITER.lock().cursor.enable_cursor(0, 15);

    // serial_println!("Hello world");
    print!("GDT        ");
    gdt::init();
    colored_print!((None, Some(Color::Green)), "OK\n");

    print!("Interrupts ");
    interrupts::init();
    colored_print!((None, Some(Color::Green)), "OK\n");
    print!("Shell      ");
    shell::init_shell();
    colored_print!((None, Some(Color::Green)), "OK\n");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable interrupts immediately — same as kernel_panic()
    unsafe {
        core::arch::asm!("cli", options(nostack, nomem));
    }

    // Capture register state while it's still warm
    let cpu_state = panic::CpuState::capture();

    // Snapshot the live stack into a static buffer
    unsafe {
        panic::save_stack();
    }

    // Print the panic report to both VGA and serial
    m_println!("\n!!! RUST PANIC !!!");
    m_println!("{}", info);

    m_println!("\nRegister snapshot:");
    cpu_state.print();

    m_println!("\nKernel stack:");
    panic::get_saved_stack().print();

    m_println!("\nSystem halted.");

    // Wipe registers and halt forever
    unsafe {
        panic::clean_registers_and_halt();
    }
}
