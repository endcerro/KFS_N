use crate::{
    keyboard::handle_keyboard_interrupt,
    m_println,
    panic::{self, CpuState},
    signals::{self, Signal},
    utils::{inb, send_eoi},
};

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
        println!("  EIP:    {:#010x}", self.eip);
        println!("  CS:     {:#06x}", self.cs);
        println!("  EFLAGS: {:#010x}", self.eflags);
        println!("  ESP:    {:#010x}", self.esp);
        println!("  SS:     {:#06x}", self.ss);
    }
}

// ---------------------------------------------------------------------------
// Kernel panic - the single path for unrecoverable faults.
//
// Sequence:
//   1. cli                     - no more interrupts
//   2. CpuState::capture()     - snapshot all registers while they're fresh
//   3. save_stack()            - copy live stack into a static buffer
//   4. print everything        - reason, stack frame, registers, stack dump
//   5. clean_registers_and_halt - zero GP regs, enter infinite hlt
// ---------------------------------------------------------------------------
pub fn kernel_panic(reason: &str, stack_frame: &InterruptStackFrame) {
    // 1. Disable interrupts immediately
    unsafe {
        core::arch::asm!("cli", options(nostack, nomem));
    }

    // 2. Capture full register state while it's still warm
    let cpu_state = CpuState::capture();

    // 3. Snapshot the kernel stack into a static buffer
    //    (safe to call here - stack is still coherent)
    unsafe {
        panic::save_stack();
    }

    // 4. Print the panic report
    m_println!("\n!!! KERNEL PANIC !!!");
    m_println!("Reason: {}", reason);

    m_println!("\nInterrupt Stack Frame (CPU-pushed):");
    stack_frame.print_debug_info();

    m_println!("\nRegister snapshot:");
    cpu_state.print();

    m_println!("\nKernel stack:");
    panic::get_saved_stack().print();

    m_println!("\nSystem halted.");

    // 5. Wipe registers and halt forever
    unsafe {
        panic::clean_registers_and_halt();
    }
}

// ---------------------------------------------------------------------------
// Timer ISR (IRQ0, vector 32)
//
// The PIT fires at ~18.2 Hz by default.  This handler does the bare
// minimum: send EOI, then conditionally schedule a TimerTick signal.
// The actual tick-counting logic lives in timer.rs and runs later
// when dispatch_pending_signals() is called from the main loop.
// ---------------------------------------------------------------------------
pub unsafe extern "x86-interrupt" fn timer_interrupt(_stack_frame: &InterruptStackFrame) {
    send_eoi(0);

    // Only enqueue a signal if someone registered a TimerTick handler.
    // When the timer demo is off, this is a no-op and the ISR is as
    // cheap as possible (just the EOI above).
    if signals::has_handler(Signal::TimerTick.as_u8()) {
        signals::schedule_signal(Signal::TimerTick.as_u8());
    }
}

// ---------------------------------------------------------------------------
// Page Fault Handler (Interrupt 14)
//
// The CPU pushes an error code with this structure:
//   Bit 0 (P)    - 0 = non-present page, 1 = protection violation
//   Bit 1 (W/R)  - 0 = read access, 1 = write access
//   Bit 2 (U/S)  - 0 = supervisor mode, 1 = user mode
//   Bit 3 (RSVD) - 1 = reserved bit set in page table entry
//   Bit 4 (I/D)  - 1 = instruction fetch (NX violation, if supported)
//
// CR2 holds the linear (virtual) address that caused the fault.
// ---------------------------------------------------------------------------
pub unsafe extern "x86-interrupt" fn page_fault(
    stack_frame: &InterruptStackFrame,
    error_code: u32,
) {
    // Read the faulting virtual address from CR2
    let faulting_address: u32;
    core::arch::asm!("mov {}, cr2", out(reg) faulting_address, options(nostack, nomem));

    // Decode the error code bits into human-readable strings
    let present = if error_code & (1 << 0) != 0 {
        "protection violation"
    } else {
        "page not present"
    };
    let operation = if error_code & (1 << 1) != 0 {
        "write"
    } else {
        "read"
    };
    let mode = if error_code & (1 << 2) != 0 {
        "user"
    } else {
        "supervisor"
    };
    let reserved = if error_code & (1 << 3) != 0 {
        " [reserved bit set]"
    } else {
        ""
    };
    let fetch = if error_code & (1 << 4) != 0 {
        " [instruction fetch]"
    } else {
        ""
    };

    m_println!("\n=== PAGE FAULT ===");
    m_println!("Faulting address (CR2): {:#010x}", faulting_address);
    m_println!("Error code: {:#06x} ({:#010b})", error_code, error_code);
    m_println!("  Cause:     {}", present);
    m_println!("  Operation: {}", operation);
    m_println!("  Mode:      {}", mode);
    if !reserved.is_empty() {
        m_println!("  {}", reserved);
    }
    if !fetch.is_empty() {
        m_println!("  {}", fetch);
    }

    // Show which PDE/PTE the fault maps to - helpful for debugging
    let pde_index = (faulting_address >> 22) as usize;
    let pte_index = ((faulting_address >> 12) & 0x3FF) as usize;
    let page_offset = faulting_address & 0xFFF;
    m_println!(
        "  PDE index: {}  PTE index: {}  Page offset: {:#05x}",
        pde_index,
        pte_index,
        page_offset
    );

    kernel_panic("Unrecoverable page fault", stack_frame);
}

// ---------------------------------------------------------------------------
// Other exception handlers
// ---------------------------------------------------------------------------

pub unsafe extern "x86-interrupt" fn divide_by_zero(stack_frame: &InterruptStackFrame) {
    kernel_panic("Divide by zero", stack_frame);
}

pub unsafe extern "x86-interrupt" fn default(stack_frame: &InterruptStackFrame) {
    println!("Unhandled interrupt fired!");
    stack_frame.print_debug_info();
    // Not necessarily fatal - some spurious interrupts can happen.
    // We log and return rather than panicking.
}

pub unsafe extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: &InterruptStackFrame) {
    let scancode = inb(0x60);
    handle_keyboard_interrupt(scancode);
    send_eoi(1);

    // Only schedule a signal if a subsystem has registered a handler.
    // Without this guard the queue fills up and drops signals when
    // nothing is consuming them (e.g. during the basic polling shell).
    if signals::has_handler(Signal::KeyboardInput.as_u8()) {
        signals::schedule_signal(Signal::KeyboardInput.as_u8());
    }
}

pub unsafe extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &InterruptStackFrame,
    _error_code: u32,
) {
    kernel_panic("Double fault", stack_frame);
}

pub unsafe extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: &InterruptStackFrame,
    error_code: u32,
) {
    println!("\n=== GENERAL PROTECTION FAULT ===");
    println!("Error code: {:#010x}", error_code);

    // If error_code is non-zero, bits [15:3] are a segment selector index
    if error_code != 0 {
        let external = error_code & 1 != 0;
        let table = (error_code >> 1) & 0x3;
        let index = (error_code >> 3) & 0x1FFF;
        let table_name = match table {
            0b00 => "GDT",
            0b01 => "IDT",
            0b10 => "LDT",
            0b11 => "IDT",
            _ => "unknown",
        };
        println!(
            "  Selector index: {} in {}{}",
            index,
            table_name,
            if external { " (external event)" } else { "" }
        );
    }

    kernel_panic("General protection fault", stack_frame);
}
