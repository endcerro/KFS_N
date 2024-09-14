//https://wiki.osdev.org/Interrupt_Descriptor_Table#Table
#![allow(dead_code)]
use core::ptr::addr_of;
use crate::gdt::define::KERNEL_CODE_SELECTOR;
use crate::{handlers, interrupts::Interrupt};
const DPL0_INTERRUPT_GATE   : u8 = 0x8E;
const DPL3_INTERRUPT_GATE   : u8 = 0xEE;
const DPL0_TRAP_GATE        : u8 = 0x8F;
const DPL3_TRAP_GATE        : u8 = 0xEF;
const DPL0_TASK_GATE        : u8 = 0x85;
const DPL3_TASK_GATE        : u8 = 0xE5;
const IDT_SIZE              : usize = 256;

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

    pub fn set_handler(&mut self, index : usize, handler : unsafe extern "x86-interrupt" fn(), selector : u16, flags : u8 ) {
        self.entries[index].set_base(handler as u32);
        self.entries[index].set_selector(selector);
        self.entries[index].set_flags(flags);
    }
}
#[repr(C, packed)]
struct Idtr {
    limit : u16,
    base : u32
}

static mut IDT : Idt = Idt::new();

pub fn load_idt() {
    let idtr = Idtr {
        limit: (size_of::<Idt>() - 1) as u16,
        base : addr_of!(IDT) as u32
    };
    unsafe {
        core::arch::asm!("lidt [{}]", in(reg) &idtr, options(readonly, nostack));
    }
}

pub fn set_interrupt_handler(index : u8, handler : unsafe extern "x86-interrupt" fn()) {
    unsafe {
        IDT.set_handler(index.into(), handler, KERNEL_CODE_SELECTOR, DPL0_INTERRUPT_GATE);
    }
}

pub fn init() {
    //Bind handlers here

    set_interrupt_handler(Interrupt::DivideError.as_u8(), handlers::divide_by_zero);
    set_interrupt_handler(Interrupt::PageFault.as_u8(), handlers::page_fault);

    for i in 0..IDT_SIZE {
        unsafe {
            if IDT.entries[i].handler_present() == false {
                set_interrupt_handler(i.try_into().unwrap(), handlers::default);
            } 
            else {
                #[cfg(feature = "verbose")]
                println!("Not setting default handler for {:?}", Interrupt::from_u8(i.try_into().unwrap()).unwrap());
            }

        }
    }

    load_idt();
    println!("IDT initialized and loaded.");
}

