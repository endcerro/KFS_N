// ---------------------------------------------------------------------------
// timer.rs — PIT tick counter and signal-driven timer demo
//
// The PIT (IRQ0, vector 32) fires at ~18.2 Hz by default (BIOS rate).
// The ISR in handlers.rs sends EOI and schedules a TimerTick signal.
//
// This module provides:
//   - A global tick counter (incremented by the signal callback)
//   - enable() / disable() to register/unregister the signal at runtime
//   - get_ticks() to read the current count
//   - is_enabled() to query state
//
// The IRQ itself is always unmasked once interrupts::init() enables it.
// What changes at runtime is whether a signal gets *queued* and
// *dispatched* — demonstrating the signal system's opt-in nature.
// ---------------------------------------------------------------------------

use crate::signals::{self, Signal};
use crate::dbg_println;
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

/// Monotonic tick counter.  Incremented by the signal callback, read
/// by anyone via get_ticks().  AtomicU32 so reads are always consistent
/// even if an interrupt fires mid-read on a non-atomic load.
static TICK_COUNT: AtomicU32 = AtomicU32::new(0);

/// Whether the timer signal handler is currently active.
static ENABLED: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// Signal callback
// ---------------------------------------------------------------------------

/// Called by dispatch_pending_signals() for every TimerTick signal.
/// Increments the tick counter and optionally displays it.
fn timer_tick_handler(_signal: u8) {
    let ticks = TICK_COUNT.fetch_add(1, Ordering::Relaxed) + 1;

    // Print to serial every ~1 second (18 ticks ≈ 1s at default PIT rate).
    // This keeps the serial log readable without flooding it.
    if ticks % 18 == 0 {
        let seconds = ticks / 18;
        serial_println!("[timer] uptime: {}s ({} ticks)", seconds, ticks);
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Register the TimerTick signal handler.  After this call, every PIT
/// interrupt that gets dispatched will increment the tick counter.
pub fn enable() {
    if ENABLED.load(Ordering::Relaxed) {
        dbg_println!("timer: already enabled");
        return;
    }
    signals::register_signal(Signal::TimerTick.as_u8(), timer_tick_handler);
    ENABLED.store(true, Ordering::Relaxed);
    println!("Timer signal enabled (ticks: {})", TICK_COUNT.load(Ordering::Relaxed));
}

/// Unregister the TimerTick signal handler.  The PIT IRQ keeps firing
/// (it's a hardware interrupt, always on), but no signals are queued
/// and the counter stops incrementing.
pub fn disable() {
    if !ENABLED.load(Ordering::Relaxed) {
        dbg_println!("timer: already disabled");
        return;
    }
    signals::unregister_signal(Signal::TimerTick.as_u8());
    ENABLED.store(false, Ordering::Relaxed);
    println!("Timer signal disabled (ticks frozen at: {})", TICK_COUNT.load(Ordering::Relaxed));
}

/// Return the current tick count.
pub fn get_ticks() -> u32 {
    TICK_COUNT.load(Ordering::Relaxed)
}

/// Return whether the timer signal is currently active.
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}