pub mod handlers;
pub mod idt;
pub mod pic;
pub mod interrupts;
pub mod define;

use interrupts::Interrupt;
use define::{DPL0_INTERRUPT_GATE, IDT_SIZE};
use crate::{gdt::define::KERNEL_CODE_SELECTOR,utils::enable_interrupts};


pub fn init() {
    //Bind handlers here

    set_interrupt_handler(Interrupt::DivideError.as_u8(), handlers::divide_by_zero);
    set_interrupt_handler_error(Interrupt::PageFault.as_u8(), handlers::page_fault);
    set_interrupt_handler(Interrupt::Keyboard.as_u8(), handlers::keyboard_interrupt);
    set_interrupt_handler_error(Interrupt::DoubleFault.as_u8(), handlers::double_fault_handler);
    set_interrupt_handler_error(Interrupt::GeneralProtectionFault.as_u8(), handlers::general_protection_fault_handler);
    for i in 0..IDT_SIZE {
        unsafe {
            if idt::IDT.entries[i].handler_present() == false {
                set_interrupt_handler(i.try_into().unwrap(), handlers::default);
            }
            else {
                #[cfg(feature = "verbose")]
                println!("Not setting default handler for {:?}", Interrupt::from_u8(i.try_into().unwrap()).unwrap());
            }
        }
    }
    // set_interrupt_handler(Interrupt::CoprocessorSegmentOverrun.as_u8(), handlers::keyboard_interrupt);

    pic::init();
    idt::load_idt();
    #[cfg(feature = "verbose")]
    println!("IDT initialized and loaded.");
    idt::configure_interrupts();
    #[cfg(feature = "verbose")]
    println!("Interrupts configured");
    unsafe {enable_interrupts(true);}
//    unsafe {
//        core::arch::asm!("int 0x21");

//    }
}

pub fn set_interrupt_handler(index : u8, handler : unsafe extern "x86-interrupt" fn(&handlers::InterruptStackFrame)) {
    unsafe {
        idt::IDT.set_handler(index.into(), handler, KERNEL_CODE_SELECTOR, DPL0_INTERRUPT_GATE);
    }
}

pub fn set_interrupt_handler_error(index : u8, handler : unsafe extern "x86-interrupt" fn(&handlers::InterruptStackFrame, u32)) {
    unsafe {
        idt::IDT.set_handler_with_errcode(index.into(), handler, KERNEL_CODE_SELECTOR, DPL0_INTERRUPT_GATE);
    }
}

