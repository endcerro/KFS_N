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
        println!("  EIP:    {:#010x}", self.eip);
        println!("  CS:     {:#06x}", self.cs);
        println!("  EFLAGS: {:#010x}", self.eflags);
        println!("  ESP:    {:#010x}", self.esp);
        println!("  SS:     {:#06x}", self.ss);
    }
}

// ---------------------------------------------------------------------------
// Kernel panic helper — prints a formatted panic message and halts.
// This disables interrupts and enters an infinite halt loop so the
// system is completely stopped.  All interrupt handlers that represent
// unrecoverable faults should call this instead of a bare `loop {}`.
// ---------------------------------------------------------------------------
pub fn kernel_panic(reason: &str, stack_frame: &InterruptStackFrame) {
    // Disable interrupts immediately so nothing else fires
    unsafe { core::arch::asm!("cli", options(nostack, nomem)); }

    println!("\n!!! KERNEL PANIC !!!");
    println!("Reason: {}", reason);
    println!("CPU State:");
    stack_frame.print_debug_info();

    // Print general-purpose registers for extra context.
    // These are the values *at this point* (inside the handler), not at
    // the exact fault instant, but they're still useful for debugging.
    unsafe {
        let eax: u32; let ebx: u32; let ecx: u32; let edx: u32;
        let esi: u32; let edi: u32; let ebp: u32;
        core::arch::asm!(
            "mov {0}, eax",
            "mov {1}, ebx",
            "mov {2}, ecx",
            "mov {3}, edx",
            "mov {4}, esi",
            "mov {5}, edi",
            "mov {6}, ebp",
            out(reg) eax,
            out(reg) ebx,
            out(reg) ecx,
            out(reg) edx,
            out(reg) esi,
            out(reg) edi,
            out(reg) ebp,
            options(nostack, nomem)
        );
        println!("Registers (handler context):");
        println!("  EAX={:#010x}  EBX={:#010x}  ECX={:#010x}  EDX={:#010x}",
            eax, ebx, ecx, edx);
        println!("  ESI={:#010x}  EDI={:#010x}  EBP={:#010x}",
            esi, edi, ebp);
    }

    println!("\nSystem halted.");

    // Infinite halt — interrupts are off so `hlt` won't wake us,
    // but loop just in case an NMI fires.
    loop {
        unsafe { core::arch::asm!("hlt", options(nostack, nomem)); }
    }
}

// ---------------------------------------------------------------------------
// Page Fault Handler (Interrupt 14)
//
// The CPU pushes an error code with this structure:
//   Bit 0 (P)    — 0 = non-present page, 1 = protection violation
//   Bit 1 (W/R)  — 0 = read access, 1 = write access
//   Bit 2 (U/S)  — 0 = supervisor mode, 1 = user mode
//   Bit 3 (RSVD) — 1 = reserved bit set in page table entry
//   Bit 4 (I/D)  — 1 = instruction fetch (NX violation, if supported)
//
// CR2 holds the linear (virtual) address that caused the fault.
// ---------------------------------------------------------------------------
pub unsafe extern "x86-interrupt" fn page_fault(stack_frame: &InterruptStackFrame, error_code: u32) {
    // Read the faulting virtual address from CR2
    let faulting_address: u32;
    core::arch::asm!("mov {}, cr2", out(reg) faulting_address, options(nostack, nomem));

    // Decode the error code bits into human-readable strings
    let present   = if error_code & (1 << 0) != 0 { "protection violation" } else { "page not present" };
    let operation = if error_code & (1 << 1) != 0 { "write" } else { "read" };
    let mode      = if error_code & (1 << 2) != 0 { "user" } else { "supervisor" };
    let reserved  = if error_code & (1 << 3) != 0 { " [reserved bit set]" } else { "" };
    let fetch     = if error_code & (1 << 4) != 0 { " [instruction fetch]" } else { "" };

    println!("\n=== PAGE FAULT ===");
    println!("Faulting address (CR2): {:#010x}", faulting_address);
    println!("Error code: {:#06x} ({:#010b})", error_code, error_code);
    println!("  Cause:     {}", present);
    println!("  Operation: {}", operation);
    println!("  Mode:      {}", mode);
    if !reserved.is_empty() { println!("  {}", reserved); }
    if !fetch.is_empty()    { println!("  {}", fetch); }

    // Show which PDE/PTE the fault maps to — helpful for debugging
    let pde_index = (faulting_address >> 22) as usize;
    let pte_index = ((faulting_address >> 12) & 0x3FF) as usize;
    let page_offset = faulting_address & 0xFFF;
    println!("  PDE index: {}  PTE index: {}  Page offset: {:#05x}",
        pde_index, pte_index, page_offset);

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
    // Not necessarily fatal — some spurious interrupts can happen.
    // We log and return rather than panicking.
}

pub unsafe extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: &InterruptStackFrame) {
    let scancode = inb(0x60);
    handle_keyboard_interrupt(scancode);
    send_eoi(1);
}

pub unsafe extern "x86-interrupt" fn double_fault_handler(stack_frame: &InterruptStackFrame, _error_code: u32) {
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
        let table    = (error_code >> 1) & 0x3;
        let index    = (error_code >> 3) & 0x1FFF;
        let table_name = match table {
            0b00 => "GDT",
            0b01 => "IDT",
            0b10 => "LDT",
            0b11 => "IDT",
            _ => "unknown",
        };
        println!("  Selector index: {} in {}{}", index, table_name,
            if external { " (external event)" } else { "" });
    }

    kernel_panic("General protection fault", stack_frame);
}