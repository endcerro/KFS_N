use core::ptr::null;

use crate::{keyboard::handle_keyboard_interrupt, utils::{inb, outb, send_eoi}};

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
    println!("Default handler error! ");
    stack_frame.print_debug_info();
    loop {}
}

pub unsafe extern "x86-interrupt" fn keyboard_interrupt(stack_frame: &InterruptStackFrame) {
    let scancode = inb(0x60);
    // println!("KEYBOARD! ");
    handle_keyboard_interrupt(scancode);
    unsafe {
        outb(crate::pic::PIC1_COMMAND, 0x20);
    }
    println!("KEYBOARD STACK FRAME : ");
    stack_frame.print_debug_info();
    // send_eoi(1);
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


// pub extern "x86-interrupt" fn interrupt_0x09_handler(stack_frame: &InterruptStackFrame) {
//     unsafe {
//         // Check if it's a real interrupt
//         crate::utils::outb(0x20, 0x0B);
//         let isr = inb(0x20);
//         if isr & (1 << 1) == 0 {  // Check if bit 1 (IRQ1) is set
//             println!("Spurious interrupt 0x09");
//             return;
//         }
//     }
//     println!("Real interrupt 0x09 occurred");
//     // Handle the interrupt...
// }