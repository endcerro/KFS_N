use crate::{keyboard::handle_keyboard_interrupt, utils::{inb, send_eoi}};

#[repr(C, align(4))]
pub struct InterruptStackFrame {
    // These are pushed by the CPU automatically
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    // These two are pushed only if there's a privilege level change (e.g., from user mode to kernel mode)
    pub esp: u32,
    pub ss: u32,
}

impl InterruptStackFrame {
     pub fn print_debug_info(&self) {
        println!("EIP: {:#x}", self.eip );
        println!("CS: {:#x}", self.cs );
        println!("EFLAGS: {:#x}", self.eflags );
        println!("ESP: {:#x}", self.esp );
        println!("SS: {:#x}", self.ss );
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
    println!("Default handler error! ");
    stack_frame.print_debug_info();
    loop {}
}

pub unsafe extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: &InterruptStackFrame) {
    let scancode = inb(0x60);
    println!("Scancode is {}", scancode);
    handle_keyboard_interrupt(scancode);
    send_eoi(1);
}

pub unsafe  extern "x86-interrupt" fn double_fault_handler(stack_frame: &InterruptStackFrame, _error_code: u32) {
    println!("EXCEPTION: DOUBLE FAULT");
    stack_frame.print_debug_info();
    loop {}
}

pub unsafe extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: &InterruptStackFrame,
    error_code: u32
) {
    println!("EXCEPTION: GENERAL PROTECTION FAULT");
    println!("Error Code: {:#x}", error_code);
    stack_frame.print_debug_info();

    loop {}
}
