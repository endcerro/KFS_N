use crate::{keyboard::handle_keyboard_interrupt, utils::{inb, send_eoi}};

#[repr(C, packed)]
pub struct InterruptStackFrame {
    // These are pushed by the CPU automatically
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    // These two are pushed only if there's a privilege level change (e.g., from user mode to kernel mode)
    pub esp: u32,
    pub ss: u32,
}

#[repr(C)]
struct AlignedInterruptStackFrame {
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub esp: u32,
    pub ss: u32,
}

impl InterruptStackFrame {
    pub fn print_debug_info(&self) {
        let aligned = 
            AlignedInterruptStackFrame {
                eip: self.eip,
                cs: self.cs,
                eflags: self.eflags,
                esp: self.esp,
                ss: self.ss,
        };
        println!("EIP: {:#x}", aligned.eip );
        println!("CS: {:#x}", aligned.cs );
        println!("EFLAGS: {:#x}", aligned.eflags );
        println!("ESP: {:#x}", aligned.esp );
        println!("SS: {:#x}", aligned.ss );
    }
}

pub unsafe extern "x86-interrupt" fn divide_by_zero(stack_frame: &InterruptStackFrame) {
    println!("Divide by zero error!");
    stack_frame.print_debug_info();
    loop {}
}

pub unsafe extern "x86-interrupt" fn page_fault(stack_frame: &InterruptStackFrame, error_code: u32) {
    println!("Page fault error!");
    println!("Error code: {:#x}", error_code);
    stack_frame.print_debug_info();
    loop {}
}

pub unsafe extern "x86-interrupt" fn default(stack_frame: &InterruptStackFrame) {
    println!("Default handler error!");
    stack_frame.print_debug_info();
    loop {}
}

pub unsafe extern "x86-interrupt" fn keyboard_interrupt(stack_frame: &InterruptStackFrame) {
    let scancode = inb(0x60);
    handle_keyboard_interrupt(scancode);
    send_eoi(1);
    loop {}
}