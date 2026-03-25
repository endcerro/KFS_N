// ---------------------------------------------------------------------------
// signals.rs - Kernel signal-callback system
//
// Two-layer design:
//
//   1. Signal table (SIGNAL_TABLE)
//      A static array of callback slots indexed by signal number.
//      register_signal() / unregister_signal() manage the bindings.
//
//   2. Signal queue (SIGNAL_QUEUE)
//      A fixed-size ring buffer.  schedule_signal() pushes a signal
//      number into the queue.  dispatch_pending_signals() drains it
//      and invokes the registered callbacks.
//
// Why a queue?  Interrupt handlers must be fast and non-reentrant.
// Instead of calling callbacks directly inside an ISR, the handler
// calls schedule_signal() (O(1), no locks needed beyond cli/sti
// which are already in effect inside the ISR), and the main loop
// calls dispatch_pending_signals() at a safe point.
//
// Signal numbers:
//   0..31   - reserved for kernel-defined signals (see Signal enum)
//   32..63  - available for user / driver defined signals
// ---------------------------------------------------------------------------

// use crate::dbg_println;

// Total number of signal slots.  Kept small - this is a kernel-only
// mechanism for now, not a full POSIX signal set.
const MAX_SIGNALS: usize = 64;

// Capacity of the pending-signal ring buffer.
// Must be a power of two so we can use mask-based wrapping.
const QUEUE_CAPACITY: usize = 64;

// ---------------------------------------------------------------------------
// Signal enum - well-known kernel signals
// ---------------------------------------------------------------------------

// Predefined kernel signal numbers.
//
// By convention signals 0–31 are kernel-reserved.
// Drivers or subsystems can use 32–63 via raw u8 values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Signal {
    // Keyboard input ready (posted by the keyboard ISR).
    KeyboardInput = 0,
    // Programmable interval timer tick.
    TimerTick = 1,
    // A page fault occurred (informational - the real handler is in the IDT,
    // but subsystems may want a notification).
    PageFault = 2,
    // Generic "something went wrong" - subsystems can use this for
    // non-fatal error notification.
    Error = 3,
    // Explicit halt request (e.g., `shutdown` shell command).
    Halt = 4,
    // Custom / user-defined signals start here.  Subsystems that need
    // their own signals should pick a number >= 32.
    UserDefined = 32,
}

impl Signal {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    // Convert a raw number to a Signal.  Returns None if out of range.
    pub fn from_u8(v: u8) -> Option<Signal> {
        match v {
            0 => Some(Signal::KeyboardInput),
            1 => Some(Signal::TimerTick),
            2 => Some(Signal::PageFault),
            3 => Some(Signal::Error),
            4 => Some(Signal::Halt),
            32 => Some(Signal::UserDefined),
            _ => None, // raw numbers are still valid for the table, just not named
        }
    }
}

// ---------------------------------------------------------------------------
// Callback type
// ---------------------------------------------------------------------------

// Signature for signal callbacks.
//
// The u8 argument is the signal number that triggered the call, so a
// single function can handle multiple signals if desired.
pub type SignalCallback = fn(u8);

// ---------------------------------------------------------------------------
// Signal table - one callback slot per signal number
// ---------------------------------------------------------------------------

// The table is just an array of Option<fn>.  `None` = no handler registered.
struct SignalTable {
    handlers: [Option<SignalCallback>; MAX_SIGNALS],
}

impl SignalTable {
    const fn new() -> Self {
        SignalTable {
            handlers: [None; MAX_SIGNALS],
        }
    }
}

static mut SIGNAL_TABLE: SignalTable = SignalTable::new();

// ---------------------------------------------------------------------------
// Signal queue - ring buffer of pending signal numbers
// ---------------------------------------------------------------------------

struct SignalQueue {
    buf: [u8; QUEUE_CAPACITY],
    // Next write position (only modified by producers - ISR context).
    head: usize,
    // Next read position (only modified by the consumer - dispatch loop).
    tail: usize,
}

impl SignalQueue {
    const fn new() -> Self {
        SignalQueue {
            buf: [0; QUEUE_CAPACITY],
            head: 0,
            tail: 0,
        }
    }

    // Number of pending entries.
    fn len(&self) -> usize {
        self.head.wrapping_sub(self.tail) & (QUEUE_CAPACITY - 1)
    }

    fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    fn is_full(&self) -> bool {
        self.len() == QUEUE_CAPACITY - 1
    }

    // Push a signal number.  Returns false if the queue is full.
    fn push(&mut self, signal: u8) -> bool {
        if self.is_full() {
            return false;
        }
        self.buf[self.head & (QUEUE_CAPACITY - 1)] = signal;
        self.head = (self.head + 1) & (QUEUE_CAPACITY - 1);
        true
    }

    // Pop the oldest pending signal.  Returns None if empty.
    fn pop(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }
        let val = self.buf[self.tail & (QUEUE_CAPACITY - 1)];
        self.tail = (self.tail + 1) & (QUEUE_CAPACITY - 1);
        Some(val)
    }
}

static mut SIGNAL_QUEUE: SignalQueue = SignalQueue::new();

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

// Register a callback for signal `signal`.
//
// Overwrites any previously registered callback for that signal.
// Pass a signal number (use `Signal::XXX.as_u8()` or a raw u8 < 64).
//
// # Example
// ```
// signals::register_signal(Signal::KeyboardInput.as_u8(), my_kb_handler);
// ```
pub fn register_signal(signal: u8, callback: SignalCallback) {
    let idx = signal as usize;
    if idx >= MAX_SIGNALS {
        dbg_println!("signals: register_signal({}) out of range", signal);
        return;
    }
    unsafe {
        SIGNAL_TABLE.handlers[idx] = Some(callback);
    }
    dbg_println!("signals: registered handler for signal {}", signal);
}

// Remove the callback for `signal`.  Future dispatches of this signal
// will be silently dropped.
pub fn unregister_signal(signal: u8) {
    let idx = signal as usize;
    if idx >= MAX_SIGNALS {
        return;
    }
    unsafe {
        SIGNAL_TABLE.handlers[idx] = None;
    }
    dbg_println!("signals: unregistered handler for signal {}", signal);
}

// Check whether a handler is currently registered for `signal`.
//
// Useful for ISRs that want to avoid the overhead of enqueuing a
// signal when nobody is listening.
pub fn has_handler(signal: u8) -> bool {
    let idx = signal as usize;
    if idx >= MAX_SIGNALS {
        return false;
    }
    unsafe { SIGNAL_TABLE.handlers[idx].is_some() }
}

// Enqueue a signal for deferred delivery.
//
// This is safe to call from interrupt context - it's O(1) and does
// not allocate.  If the queue is full the signal is dropped and a
// warning is logged to serial.
//
// Typical call site: inside an ISR, after doing the minimum hardware
// work (read scancode, send EOI, etc.).
pub fn schedule_signal(signal: u8) {
    if (signal as usize) >= MAX_SIGNALS {
        dbg_println!("signals: schedule_signal({}) out of range", signal);
        return;
    }
    unsafe {
        if !SIGNAL_QUEUE.push(signal) {
            // Queue full - drop the signal.  This is a soft error;
            // losing a timer tick or an extra keyboard event is
            // acceptable.  Losing a Halt is not ideal but the queue
            // would need to be absurdly backed up.
            dbg_println!("signals: queue full, dropped signal {}", signal);
        }
    }
}

// Drain the signal queue and invoke registered callbacks.
//
// Call this from a safe, non-interrupt context - typically the main
// loop or the shell idle loop.  Interrupts are briefly disabled
// while we pop each entry to avoid racing with ISR producers, then
// re-enabled for the actual callback invocation.
//
// Returns the number of signals dispatched.
pub fn dispatch_pending_signals() -> usize {
    let mut count = 0usize;

    loop {
        // --- critical section: pop one entry with interrupts off ---
        let sig: Option<u8>;
        unsafe {
            core::arch::asm!("cli", options(nostack, nomem));
            sig = SIGNAL_QUEUE.pop();
            core::arch::asm!("sti", options(nostack, nomem));
        }

        let signal = match sig {
            Some(s) => s,
            None => break, // queue empty
        };

        // --- invoke callback with interrupts enabled ---
        let cb = unsafe { SIGNAL_TABLE.handlers[signal as usize] };
        if let Some(callback) = cb {
            callback(signal);
            count += 1;
        }
        // If no handler is registered the signal is silently consumed.
    }
    count
}

// Check whether any signals are pending without consuming them.
pub fn has_pending_signals() -> bool {
    unsafe { !SIGNAL_QUEUE.is_empty() }
}

// Return how many signals are currently queued.
pub fn pending_count() -> usize {
    unsafe { SIGNAL_QUEUE.len() }
}

// Diagnostic: print the signal table to VGA/serial.
pub fn print_signal_table() {
    println!("Signal table ({} slots):", MAX_SIGNALS);
    for i in 0..MAX_SIGNALS {
        unsafe {
            if let Some(cb) = SIGNAL_TABLE.handlers[i] {
                // Print the function pointer address - helpful for debugging
                let name = match Signal::from_u8(i as u8) {
                    Some(s) => match s {
                        Signal::KeyboardInput => "KeyboardInput",
                        Signal::TimerTick => "TimerTick",
                        Signal::PageFault => "PageFault",
                        Signal::Error => "Error",
                        Signal::Halt => "Halt",
                        Signal::UserDefined => "UserDefined",
                    },
                    None => "(unnamed)",
                };
                println!("  [{}] {} -> {:#010x}", i, name, cb as usize);
            }
        }
    }
}
