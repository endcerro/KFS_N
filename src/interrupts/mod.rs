use core::arch::asm;
mod structures;

#[no_mangle]
pub extern "C" fn exception_handler(frame: &structures::InterruptStackFrame) {
    println!("Interrupt: {}", frame.interrupt_number);
    println!("Error Code: {}", frame.error_code);
    // Handle the interrupt based on its number
    match frame.interrupt_number {
        13 => general_protection_fault_handler(frame),
        0..=31 => handle_exception(frame),
        0x20 => timer_interrupt_handler(),
        0x21 => keyboard_interrupt_handler(),
        _ => unknown_interrupt_handler(frame.interrupt_number),
    }

    if frame.interrupt_number >= 0x20 && frame.interrupt_number < 0x30 {
        unsafe {
            // Send End of Interrupt (EOI) signal
            if frame.interrupt_number >= 0x28 {
                // Send EOI to PIC2 (slave)
                structures::port_write(0xA0, 0x20);
            }
            // Send EOI to PIC1 (master)
            structures::port_write(0x20, 0x20);
        }
    }
    println!("WE GOT AN INTERRUP BRO");
    unsafe {
        asm!("cli","hlt");
    }
}

#[no_mangle]
pub extern "C" fn general_protection_fault_handler(frame: &structures::InterruptStackFrame) {
    println!("EXCEPTION: GENERAL PROTECTION FAULT");
    println!("Error Code: 0x{:x}", frame.error_code);
    println!("EIP: 0x{:x}, CS: 0x{:x}, EFLAGS: 0x{:x}", frame.eip, frame.cs, frame.eflags);
    println!("ESP: 0x{:x}, SS: 0x{:x}", frame.user_esp, frame.ss);
    println!("EAX: 0x{:x}, EBX: 0x{:x}, ECX: 0x{:x}, EDX: 0x{:x}", 
             frame.eax, frame.ebx, frame.ecx, frame.edx);
    println!("ESI: 0x{:x}, EDI: 0x{:x}, EBP: 0x{:x}", 
             frame.esi, frame.edi, frame.ebp);
    println!("DS: 0x{:x}, ES: 0x{:x}, FS: 0x{:x}, GS: 0x{:x}", 
             frame.ds, frame.es, frame.fs, frame.gs);
    
    // Analyze error code
    if frame.error_code != 0 {
        let table = if frame.error_code & 0x4 != 0 { "IDT" } else if frame.error_code & 0x2 != 0 { "LDT" } else { "GDT" };
        let index = frame.error_code >> 3;
        println!("Selector Error in {}, index {}", table, index);
    }

    // Halt the CPU
    loop {
        unsafe { asm!("hlt") };
    }
}
fn handle_exception(frame: &structures::InterruptStackFrame) {
    println!("Exception: {} (Error Code: {})", frame.interrupt_number, frame.error_code);
    println!("EIP: 0x{:x}, CS: 0x{:x}, EFLAGS: 0x{:x}", frame.eip, frame.cs, frame.eflags);
    println!("EAX: 0x{:x}, EBX: 0x{:x}, ECX: 0x{:x}, EDX: 0x{:x}", 
             frame.eax, frame.ebx, frame.ecx, frame.edx);
    println!("ESP: 0x{:x}, EBP: 0x{:x}, ESI: 0x{:x}, EDI: 0x{:x}", 
             frame.esp, frame.ebp, frame.esi, frame.edi);

    // Halt the CPU for exceptions
    unsafe { asm!("cli", "hlt") };
}

fn timer_interrupt_handler() {
    // Handle timer interrupt
    println!("Timer interrupt");
}

fn keyboard_interrupt_handler() {
    // Handle keyboard interrupt
    println!("Keyboard interrupt");
    // Read from keyboard I/O port
    let scancode: u8 = unsafe { structures::port_read(0x60) };
    // Process scancode
}

fn unknown_interrupt_handler(interrupt_number: u32) {
    println!("Unknown interrupt: {:x}", interrupt_number);
}

// Unsafe function to read from an I/O port


pub fn init() {
    structures::idt_init();
}