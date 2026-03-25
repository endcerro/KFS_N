pub mod define;
pub mod handlers;
pub mod idt;
pub mod interrupts;
pub mod pic;

use crate::dbg_println;
// use crate::interrupts::handlers::timer_interrupt;
use crate::{gdt::define::KERNEL_CODE_SELECTOR, utils::enable_interrupts};
use define::{DPL0_INTERRUPT_GATE, IDT_SIZE};
use interrupts::Interrupt;

pub fn init() {
    //Bind handlers here
    set_interrupt_handler(Interrupt::DivideError.as_u8(), handlers::divide_by_zero);
    set_interrupt_handler_error(Interrupt::PageFault.as_u8(), handlers::page_fault);
    set_interrupt_handler(Interrupt::Keyboard.as_u8(), handlers::keyboard_interrupt);
    set_interrupt_handler_error(
        Interrupt::DoubleFault.as_u8(),
        handlers::double_fault_handler,
    );
    set_interrupt_handler_error(
        Interrupt::GeneralProtectionFault.as_u8(),
        handlers::general_protection_fault_handler,
    );
    set_interrupt_handler(
        Interrupt::ProgrammableInterruptTimer.as_u8(),
        handlers::timer_interrupt,
    );
    for i in 0..IDT_SIZE {
        unsafe {
            if idt::IDT.entries[i].handler_present() == false {
                set_interrupt_handler(i.try_into().unwrap(), handlers::default);
            } else {
                dbg_println!(
                    "Not setting default handler for {:?}",
                    Interrupt::from_u8(i.try_into().unwrap()).unwrap()
                );
            }
        }
    }
    pic::init();
    idt::load_idt();
    dbg_println!("IDT initialized and loaded.");
    idt::configure_interrupts();
    dbg_println!("Interrupts configured");
    unsafe {
        enable_interrupts(true);
    }
}

pub fn set_interrupt_handler(
    index: u8,
    handler: unsafe extern "x86-interrupt" fn(&handlers::InterruptStackFrame),
) {
    unsafe {
        idt::IDT.set_handler(
            index.into(),
            handler,
            KERNEL_CODE_SELECTOR,
            DPL0_INTERRUPT_GATE,
        );
    }
}

pub fn set_interrupt_handler_error(
    index: u8,
    handler: unsafe extern "x86-interrupt" fn(&handlers::InterruptStackFrame, u32),
) {
    unsafe {
        idt::IDT.set_handler_with_errcode(
            index.into(),
            handler,
            KERNEL_CODE_SELECTOR,
            DPL0_INTERRUPT_GATE,
        );
    }
}
