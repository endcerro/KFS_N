use core::arch::asm;

const IDT_MAX_DESCRIPTORS : usize = 256;

#[derive(Clone, Copy)]
#[repr(packed)]
struct IdtEntryT {
    isr_low : u16,
    kernel_cs : u16,
    reserved : u8,
    attributes : u8,
    isr_high : u16
}

impl IdtEntryT {
    const fn default() -> IdtEntryT {
        IdtEntryT {
            isr_low : 0,
            kernel_cs : 0,
            reserved : 0,
            attributes : 0,
            isr_high : 0
        }
    }
}


#[repr(packed)]
struct Idtr {
     limit : u16,
     base : u32,
    }

static mut IDT : [IdtEntryT; IDT_MAX_DESCRIPTORS] = [IdtEntryT::default(); IDT_MAX_DESCRIPTORS];

static mut IDTR : Idtr = Idtr{limit : 0,base : 0};

#[no_mangle]
pub extern "C" fn exception_handler() {
    println!("WE GOT AN INTERRUP BRO");
    unsafe {
        asm!("cli","hlt");
    }
}

pub fn idt_set_descriptor( vector: usize, isr_ptr: usize, flags: u8 )
{
    unsafe {
        let descriptor : &mut IdtEntryT = &mut IDT[vector];
        descriptor.isr_low = (isr_ptr & 0xFFFF) as u16;
        descriptor.kernel_cs = 0x08;
        descriptor.attributes = flags;
        descriptor.isr_high = (isr_ptr >> 16) as u16;
        descriptor.reserved = 0;
    }
}

static mut VECTORS : [bool; IDT_MAX_DESCRIPTORS] = [false; IDT_MAX_DESCRIPTORS];

extern "C" {
    static mut isr_stub_table: *mut usize;
}

pub fn idt_init()
{
    unsafe {
        IDTR.base = (&(IDT[0]) as *const _) as u32;
        IDTR.limit = (size_of::<IdtEntryT>() * IDT_MAX_DESCRIPTORS - 1) as u16;

        for vector in 0..32{
            idt_set_descriptor(vector, isr_stub_table.wrapping_add(vector) as usize, 0x8E);
            VECTORS[vector] = true;
        }
        asm!(
            "lidt [{}]",
            in(reg) core::ptr::addr_of!(IDTR),
            options(readonly, nostack, preserves_flags)
        );

        asm!("sti", options(readonly, nostack, preserves_flags));
    }
}