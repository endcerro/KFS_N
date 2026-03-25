// ---------------------------------------------------------------------------
// panic.rs — Pre-panic state capture and cleanup interfaces
//
// Provides three capabilities that handlers use before halting:
//   1. CpuState::capture()   — snapshot all GP/segment/control registers
//   2. save_stack()          — copy a bounded region of the live kernel stack
//                              into a static buffer (survives stack corruption)
//   3. clean_registers()     — zero all GP registers right before halt
//                              (prevents information leakage)
//
// These are building blocks: kernel_panic() in handlers.rs orchestrates them.
// ---------------------------------------------------------------------------
use crate::m_print;
use crate::m_println;
/// Maximum number of stack bytes we snapshot into the static buffer.
/// 512 bytes = 128 dwords, enough to capture a useful call chain without
/// blowing our memory budget.
const STACK_SAVE_SIZE: usize = 512;

/// Static buffer for the stack snapshot.  Written once during a panic,
/// never freed.  Keeps the data available even if the original stack
/// memory becomes inaccessible (e.g. after a double fault overwrites it).
static mut SAVED_STACK: StackSnapshot = StackSnapshot {
    buf: [0u8; STACK_SAVE_SIZE],
    esp: 0,
    stack_top: 0,
    len: 0,
    valid: false,
};

// ---------------------------------------------------------------------------
// CpuState — full register snapshot
// ---------------------------------------------------------------------------

/// Complete snapshot of the i386 register file at a point in time.
/// Populated by inline assembly so the values are as close to the
/// fault instant as the calling convention allows.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CpuState {
    // General-purpose registers
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    pub ebp: u32,
    pub esp: u32,

    // Segment registers
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
    pub ss: u16,

    // Control registers (read-only for diagnostics)
    pub cr0: u32,
    pub cr2: u32, // faulting address on page faults
    pub cr3: u32, // page directory base
}

impl CpuState {
    pub const fn empty() -> Self {
        CpuState {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
            esi: 0,
            edi: 0,
            ebp: 0,
            esp: 0,
            cs: 0,
            ds: 0,
            es: 0,
            fs: 0,
            gs: 0,
            ss: 0,
            cr0: 0,
            cr2: 0,
            cr3: 0,
        }
    }

    /// Snapshot every accessible register right now.
    ///
    /// **Important**: the values of EAX/ECX/EDX may reflect the compiler's
    /// register allocation for this function rather than the true fault-time
    /// values.  EBX/ESI/EDI/EBP are callee-saved so they're more reliable.
    /// For the most accurate GP values, prefer the InterruptStackFrame that
    /// the CPU pushes automatically (EIP, CS, EFLAGS, ESP, SS).
    pub fn capture() -> Self {
        let mut state = CpuState::empty();
        unsafe {
            // GP registers — captured in one asm block to minimise clobbering
            core::arch::asm!(
                "mov [{s} + 0],  eax",
                "mov [{s} + 4],  ebx",
                "mov [{s} + 8],  ecx",
                "mov [{s} + 12], edx",
                "mov [{s} + 16], esi",
                "mov [{s} + 20], edi",
                "mov [{s} + 24], ebp",
                "mov [{s} + 28], esp",
                s = in(reg) &mut state as *mut CpuState as u32,
                // We clobber nothing extra — the mov instructions read
                // the live register values before the compiler touches them.
            );

            // Segment registers — must go through a GPR on i386
            core::arch::asm!("mov {:x}, cs", out(reg) state.cs);
            core::arch::asm!("mov {:x}, ds", out(reg) state.ds);
            core::arch::asm!("mov {:x}, es", out(reg) state.es);
            core::arch::asm!("mov {:x}, fs", out(reg) state.fs);
            core::arch::asm!("mov {:x}, gs", out(reg) state.gs);
            core::arch::asm!("mov {:x}, ss", out(reg) state.ss);

            // Control registers
            core::arch::asm!("mov {}, cr0", out(reg) state.cr0, options(nostack, nomem));
            core::arch::asm!("mov {}, cr2", out(reg) state.cr2, options(nostack, nomem));
            core::arch::asm!("mov {}, cr3", out(reg) state.cr3, options(nostack, nomem));
        }
        state
    }

    /// Pretty-print the full register dump to VGA + serial.
    pub fn print(&self) {
        m_println!("  --- General Purpose Registers ---");
        m_println!(
            "  EAX={:#010x}  EBX={:#010x}  ECX={:#010x}  EDX={:#010x}",
            self.eax,
            self.ebx,
            self.ecx,
            self.edx
        );
        m_println!(
            "  ESI={:#010x}  EDI={:#010x}  EBP={:#010x}  ESP={:#010x}",
            self.esi,
            self.edi,
            self.ebp,
            self.esp
        );
        m_println!("  --- Segment Registers ---");
        m_println!(
            "  CS={:#06x}  DS={:#06x}  ES={:#06x}  FS={:#06x}  GS={:#06x}  SS={:#06x}",
            self.cs,
            self.ds,
            self.es,
            self.fs,
            self.gs,
            self.ss
        );
        m_println!("  --- Control Registers ---");
        m_println!(
            "  CR0={:#010x}  CR2={:#010x}  CR3={:#010x}",
            self.cr0,
            self.cr2,
            self.cr3
        );
    }
}

// ---------------------------------------------------------------------------
// Stack snapshot
// ---------------------------------------------------------------------------

/// A bounded copy of the kernel stack at panic time.
pub struct StackSnapshot {
    buf: [u8; STACK_SAVE_SIZE],
    /// ESP value when the snapshot was taken.
    pub esp: u32,
    /// Top of the kernel stack (highest valid address).
    pub stack_top: u32,
    /// How many bytes were actually copied (may be < STACK_SAVE_SIZE if
    /// the live stack was smaller).
    pub len: usize,
    /// Set to true once a valid snapshot has been stored.
    pub valid: bool,
}

impl StackSnapshot {
    /// Print the saved stack as a hexdump.
    pub fn print(&self) {
        if !self.valid {
            m_println!("  (no stack snapshot available)");
            return;
        }
        m_println!(
            "  Stack snapshot: {} bytes from ESP={:#010x}  TOP={:#010x}",
            self.len,
            self.esp,
            self.stack_top
        );
        // Print as rows of 16 bytes, matching the hexdump convention
        let mut offset = 0usize;
        while offset < self.len {
            let addr = self.esp as usize + offset;
            let row_len = core::cmp::min(16, self.len - offset);
            // Address column
            m_print!("  {:#010x}  ", addr);
            // Hex bytes
            for i in 0..row_len {
                if i == 8 {
                    m_print!(" ");
                }
                m_print!("{:02x} ", self.buf[offset + i]);
            }
            // Pad if short row
            for _ in row_len..16 {
                m_print!("   ");
            }
            // ASCII column
            m_print!(" |");
            for i in 0..row_len {
                let b = self.buf[offset + i];
                if b >= 0x20 && b <= 0x7e {
                    m_print!("{}", b as char);
                } else {
                    m_print!(".");
                }
            }
            m_println!("|");
            offset += 16;
        }
    }
}

/// Copy up to STACK_SAVE_SIZE bytes of the live kernel stack into
/// the static SAVED_STACK buffer.
///
/// Call this early in a panic path — before any further stack usage
/// can overwrite the interesting frames.
///
/// # Safety
/// Reads raw memory between ESP and `stack_top`.  Must only be
/// called when the kernel stack is still in a consistent-enough
/// state to be read (i.e. not from a double-fault that trashed it).
pub unsafe fn save_stack() {
    let esp: u32;
    let top: u32;
    core::arch::asm!(
        "mov {esp}, esp",
        "lea {top}, [stack_top]",
        esp = out(reg) esp,
        top = out(reg) top,
        options(nostack, nomem),
    );

    if esp >= top {
        // Stack pointer is at or above top — nothing useful to copy.
        SAVED_STACK.valid = false;
        return;
    }

    let available = (top - esp) as usize;
    let copy_len = core::cmp::min(available, STACK_SAVE_SIZE);

    // Byte-by-byte copy — we don't trust memcpy here because the
    // allocator or other subsystems might be in a broken state.
    let src = esp as *const u8;
    for i in 0..copy_len {
        SAVED_STACK.buf[i] = *src.add(i);
    }

    SAVED_STACK.esp = esp;
    SAVED_STACK.stack_top = top;
    SAVED_STACK.len = copy_len;
    SAVED_STACK.valid = true;
}

/// Return a reference to the saved stack snapshot (may be invalid if
/// save_stack() hasn't been called or the stack was empty).
pub fn get_saved_stack() -> &'static StackSnapshot {
    unsafe { &SAVED_STACK }
}

// ---------------------------------------------------------------------------
// Register cleaning
// ---------------------------------------------------------------------------

/// Zero all general-purpose registers.
///
/// Called right before the final halt loop to prevent information
/// leakage (e.g. crypto keys, user data lingering in registers).
/// After this call the only safe thing to do is `hlt` in a loop —
/// any Rust code that touches local variables will immediately break
/// because EBP/ESP are gone.
///
/// # Safety
/// Destroys the entire GP register set including EBP and ESP.
/// Must be the very last thing before the halt loop, and that halt
/// loop must be in the same inline asm block.
pub unsafe fn clean_registers_and_halt() -> ! {
    core::arch::asm!(
        // Zero every general-purpose register
        "xor eax, eax",
        "xor ebx, ebx",
        "xor ecx, ecx",
        "xor edx, edx",
        "xor esi, esi",
        "xor edi, edi",
        "xor ebp, ebp",
        // NOTE: we do NOT zero ESP — the `hlt` instruction still needs
        // a valid stack pointer in case an NMI fires.  The stack contents
        // have already been wiped by save_stack / are no longer relevant.

        // Infinite halt loop — interrupts are off (cli was called earlier),
        // so hlt won't return.  The loop guards against NMIs.
        "2:",
        "hlt",
        "jmp 2b",
        options(noreturn, nostack),
    );
}
