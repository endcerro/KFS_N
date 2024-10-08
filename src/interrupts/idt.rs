//https://wiki.osdev.org/Interrupt_Descriptor_Table#Table
#![allow(dead_code)]
// use core::ptr::addr_of;
// use crate::gdt::define::KERNEL_CODE_SELECTOR;
// use crate::pic::{self, set_irq_state};
// use crate::{handlers, interrupts::*};

use core::ptr::addr_of;
use super::pic::set_irq_state;

use super::{define::IDT_SIZE, handlers, interrupts::Interrupt};


#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    base_low : u16,         // The lower 16 bits of the address to jump to when this interrupt fires.
    segment_selector : u16, // Kernel segment selector.
    zero : u8,              // This must always be zero.
    flags : u8,             // More flags. See documentation.
    base_high : u16         // The upper 16 bits of the address to jump to.
}
impl IdtEntry {
    pub const fn new() -> Self {
        IdtEntry {
            base_low : 0,
            segment_selector : 0,
            zero : 0,
            flags : 0,
            base_high : 0
        }
    }
    pub fn set_base(&mut self, base : u32) {
        self.base_low = (base & 0xFFFF) as u16;
        self.base_high = ((base >> 16) & 0xFFFF) as u16;
    }
    pub fn set_selector(&mut self, selector : u16) {
        self.segment_selector = selector;
    }
    pub fn set_flags(&mut self, flags : u8) {
        self.flags = flags;
    }
    pub fn handler_present(&self) -> bool {
        self.base_low != 0 && self.base_high != 0
    }
}

pub struct Idt {
    pub entries: [IdtEntry; IDT_SIZE]
}

impl Idt {
    pub const fn new() -> Self {
        Idt { entries : [IdtEntry::new(); IDT_SIZE] }
    }

    pub fn set_handler(&mut self, index : usize, handler : unsafe extern "x86-interrupt" fn(&handlers::InterruptStackFrame), selector : u16, flags : u8 ) {
        self.entries[index].set_base(handler as usize as u32);
        self.entries[index].set_selector(selector);
        self.entries[index].set_flags(flags);
    }
    pub fn set_handler_with_errcode(&mut self, index : usize, handler : unsafe extern "x86-interrupt" fn(&handlers::InterruptStackFrame, u32), selector : u16, flags : u8 ) {
        self.entries[index].set_base(handler as usize as u32);
        self.entries[index].set_selector(selector);
        self.entries[index].set_flags(flags);
    }
}
#[repr(C, packed)]
struct Idtr {
    limit : u16,
    base : u32
}

pub fn load_idt() {
    let idtr = Idtr {
        limit: (size_of::<Idt>() - 1) as u16,
        base : addr_of!(IDT) as u32
    };
    unsafe {
        core::arch::asm!("lidt [{}]", in(reg) &idtr, options(readonly, nostack));
    }
}

pub fn configure_interrupts() {
    for i in 32..48 {
        set_irq_state(Interrupt::from_u8(i).expect("Configure int error"), false);
    }
    // set_irq_state(Interrupt::Timer, true);
    set_irq_state(Interrupt::Keyboard, true);
    // set_irq_state(Interrupt::CascadeForPIC2, true); // Always enable this for PIC2
}

pub static mut IDT : Idt = Idt::new();