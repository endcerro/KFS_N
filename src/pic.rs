use crate::{interrupts::Interrupt, utils::{inb, outb}};
// PIC ports
pub const PIC1_COMMAND: u16 = 0x20;
pub const PIC1_DATA: u16 = 0x21;
pub const PIC2_COMMAND: u16 = 0xA0;
pub const PIC2_DATA: u16 = 0xA1;

// Initialization Command Words
pub const ICW1_INIT: u8 = 0x11;
pub const ICW4_8086: u8 = 0x01;

// New interrupt vector offsets for remapped PICs
pub const PIC1_OFFSET: u8 = 0x20; // IRQs 0-7 mapped to interrupts 0x20-0x27
pub const PIC2_OFFSET: u8 = 0x28; // IRQs 8-15 mapped to interrupts 0x28-0x2F

// PIC configuration
pub const PIC1_ICW2: u8 = PIC1_OFFSET;
pub const PIC2_ICW2: u8 = PIC2_OFFSET;
pub const PIC1_ICW3: u8 = 4; // 0000 0100 - Slave PIC at IRQ2
pub const PIC2_ICW3: u8 = 2; // Slave PIC cascade identity

// IRQ masks
pub const PIC1_MASK_ALL_EXCEPT_KEYBOARD: u8 = 0xFD; // 1111 1101
pub const PIC2_MASK_ALL: u8 = 0xFF; // 1111 1111

pub fn init() {
    // Remap PIC
    outb(0x20, 0x11);
    outb(0xA0, 0x11);
    outb(0x21, 0x20);
    outb(0xA1, 0x28);
    outb(0x21, 0x04);
    outb(0xA1, 0x02);
    outb(0x21, 0x01);
    outb(0xA1, 0x01);

    // Unmask keyboard interrupt (IRQ1) and cascade interrupt (IRQ2)
    // let mask = inb(0x21) & 0xF9; // 1111 1001
    outb(0x21, 0xFD);
    outb(0xA1, 0xFF);
}

pub fn set_irq_state(interrupt: Interrupt, enabled: bool) {
    if (interrupt as u8) < 32 {
        // These are CPU exceptions, not maskable through the PIC
        return;
    }

    let irq = (interrupt as u8) - 32;
    let (port, irq_bit) = if irq < 8 {
        (PIC1_DATA, irq)
    } else {
        (PIC2_DATA, irq - 8)
    };

    let mut mask = inb(port);
    if enabled {
        mask &= !(1 << irq_bit);
    } else {
        mask |= 1 << irq_bit;
    }
    outb(port, mask);
}
