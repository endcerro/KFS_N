use core::arch::asm;

pub const IDT_MAX_DESCRIPTORS : usize = 256;

extern "C" {
    static mut isr_stub_table: [u32; 32];
}
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntryT {
    pub isr_low : u16,
    pub kernel_cs : u16,
    pub reserved : u8,
    pub attributes : u8,
    pub isr_high : u16
}

impl IdtEntryT {
    const fn new(isr: u32, selector: u16, flags: u8) -> Self {
        IdtEntryT {
            isr_low : (isr & 0xFFFF) as u16,
            kernel_cs : selector,
            reserved : 0,
            attributes: flags,
            isr_high: ((isr >> 16) & 0xFFFF) as u16
        }
    }
}


#[repr(packed)]
pub struct Idtr {
     limit : u16,
     base : u32,
    }

pub static mut IDT : [IdtEntryT; IDT_MAX_DESCRIPTORS] = [IdtEntryT::new(0,0,0); IDT_MAX_DESCRIPTORS];

pub static mut IDTR : Idtr = Idtr{limit : 0,base : 0};

static mut VECTORS : [bool; IDT_MAX_DESCRIPTORS] = [false; IDT_MAX_DESCRIPTORS];

/*
This struct contains information that will be pushed onto the stack in case of an interrupt
These will be useful to restore the context from before said interrupt */
#[repr(C)]
pub struct InterruptStackFrame {
    pub gs: u32,
    pub fs: u32,
    pub es: u32,
    pub ds: u32,
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub interrupt_number: u32,
    pub error_code: u32,
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub user_esp: u32,
    pub ss: u32,
}

extern "C"  {
    fn load_idt(idtr: *const Idtr);
}

pub unsafe fn port_read(port: u16) -> u8 {
    let result: u8;
    asm!("in al, dx", out("al") result, in("dx") port, options(nomem, nostack));
    result
}
pub unsafe fn port_write(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}
fn remap_pic() {
    unsafe {
        // Initialize PIC1
        port_write(0x20, 0x11);
        port_write(0x21, 0x20);
        port_write(0x21, 0x04);
        port_write(0x21, 0x01);

        // Initialize PIC2
        port_write(0xA0, 0x11);
        port_write(0xA1, 0x28);
        port_write(0xA1, 0x02);
        port_write(0xA1, 0x01);

        // Unmask all interrupts
        port_write(0x21, 0x0);
        port_write(0xA1, 0x0);
    }
}
pub fn idt_init()
{
    unsafe {
        for (i, &isr_addr) in isr_stub_table.iter().enumerate() {
            IDT[i] = IdtEntryT::new(
                isr_addr,
        0x08, // Kernel code segment selector
            0x8E // Present, Ring 0, 32-bit Interrupt Gate
            );
        }

        IDTR = Idtr {
            limit : (size_of::<[IdtEntryT; IDT_MAX_DESCRIPTORS]>() - 1) as u16,
        base : unsafe { &IDT as *const _ as u32}
    };
    load_idt(&IDTR);
    remap_pic();
    asm!("sti", options(readonly, nostack, preserves_flags));
    }
}