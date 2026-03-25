// ---------------------------------------------------------------------------
// timer.rs — PIT tick counter and signal-driven timer display
//
// The PIT (IRQ0, vector 32) fires at ~18.2 Hz by default (BIOS rate).
// The ISR in handlers.rs sends EOI and schedules a TimerTick signal.
//
// Display modes (independent, can be combined):
//   - counter:   raw tick count in top-right corner
//   - uptime:    HH:MM:SS formatted uptime in top-right corner
//   - heartbeat: alternating character pulsing in top-right corner
//
// All three write to VGA row 0 at the right edge.  They share the
// same signal handler — the callback checks which modes are active
// and renders accordingly.
//
// The underlying tick counter always runs when any mode is active.
// ---------------------------------------------------------------------------

use crate::dbg_println;
use crate::signals::{self, Signal};
use crate::vga::{
    self, get_current_colors, vga_clear_region, vga_write_at, Color, ColorCode, ScreenCharacter,
    VGA_BUFFER_WIDTH,
};
use core::sync::atomic::{AtomicU32, AtomicU8, Ordering};

/// Approximate PIT frequency — BIOS default is 1193182/65536 ≈ 18.2 Hz.
/// Used to convert ticks to seconds for uptime display.
const PIT_HZ: u32 = 18;

/// Monotonic tick counter.  Always incremented while any mode is active.
static TICK_COUNT: AtomicU32 = AtomicU32::new(0);

/// Bitmask of active display modes.  0 = nothing active (handler not
/// registered).  Individual bits correspond to DisplayMode values.
static ACTIVE_MODES: AtomicU8 = AtomicU8::new(0);

// Bit positions for each mode
const MODE_COUNTER: u8 = 1 << 0;
const MODE_UPTIME: u8 = 1 << 1;
const MODE_HEARTBEAT: u8 = 1 << 2;

// ---------------------------------------------------------------------------
// VGA direct-write helpers
//
// We write directly to the VGA buffer instead of going through the
// Writer/lock path.  This avoids disturbing the cursor position and
// works safely from the signal callback (which runs with interrupts
// enabled but outside any lock).
// ---------------------------------------------------------------------------

/// Row 0 is the top of the screen — we'll use it as a status bar area.
const STATUS_ROW: usize = 0;

/// Color for the status display: yellow on black, distinct from normal text.
// const STATUS_COLOR: ColorCode = ColorCode::new(Color::Yellow, Color::Black);

/// Clear a region of VGA row with spaces.
// fn vga_clear_region(row: usize, col_start: usize, col_end: usize, color: ColorCode) {
//     let buf = unsafe { &mut *(vga::VGA_BUFFER_ADDR as *mut vga::Buffer) };
//     for c in col_start..col_end {
//         if c >= VGA_BUFFER_WIDTH {
//             break;
//         }
//         buf.chars[row][c] = ScreenCharacter {
//             ascii_value: b' ',
//             color,
//         };
//     }
// }

// ---------------------------------------------------------------------------
// Signal callback
// ---------------------------------------------------------------------------

/// Called by dispatch_pending_signals() for every TimerTick signal.
/// Increments the tick counter, then renders whichever display modes
/// are active.
fn timer_tick_handler(_signal: u8) {
    let ticks = TICK_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    let modes = ACTIVE_MODES.load(Ordering::Relaxed);

    // Each mode writes to a different region of row 0, right-aligned:
    //   heartbeat: col 79        (1 char)
    //   uptime:    col 69..77    (8 chars "HH:MM:SS")
    //   counter:   col 58..67    (up to 10 digits)
    //
    // Laid out:  [... normal text ...] [counter   ] [uptime  ] [beat]

    let current_color: ColorCode = ColorCode::new(get_current_colors().0, get_current_colors().1);
    if modes & MODE_HEARTBEAT != 0 {
        // Alternate between '*' and ' ' every ~9 ticks (~0.5s)
        let ch = if (ticks / 9) % 2 == 0 { b'*' } else { b' ' };
        vga_write_at(STATUS_ROW, 79, &[ch], current_color);
    }

    if modes & MODE_UPTIME != 0 {
        let total_secs = ticks / PIT_HZ;
        let hours = (total_secs / 3600) % 100; // cap at 99h
        let minutes = (total_secs / 60) % 60;
        let seconds = total_secs % 60;

        // Format HH:MM:SS into a fixed buffer — no alloc needed
        let mut buf = [b'0'; 8]; // "00:00:00"
        buf[0] = b'0' + (hours / 10) as u8;
        buf[1] = b'0' + (hours % 10) as u8;
        buf[2] = b':';
        buf[3] = b'0' + (minutes / 10) as u8;
        buf[4] = b'0' + (minutes % 10) as u8;
        buf[5] = b':';
        buf[6] = b'0' + (seconds / 10) as u8;
        buf[7] = b'0' + (seconds % 10) as u8;

        vga_write_at(STATUS_ROW, 70, &buf, current_color);
    }

    if modes & MODE_COUNTER != 0 {
        // Format the raw tick count right-aligned into a 10-char field.
        let mut buf = [b' '; 10];
        let mut n = ticks;
        let mut i = 9;
        loop {
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
            if n == 0 || i == 0 {
                break;
            }
            i -= 1;
        }
        vga_write_at(STATUS_ROW, 58, &buf, current_color);
    }
}

// ---------------------------------------------------------------------------
// Internal: manage signal registration
//
// The signal handler is registered when transitioning from 0 active
// modes to ≥1, and unregistered when going back to 0.
// ---------------------------------------------------------------------------

fn update_signal_registration(old_modes: u8, new_modes: u8) {
    let current_color: ColorCode = ColorCode::new(get_current_colors().0, get_current_colors().1);
    if old_modes == 0 && new_modes != 0 {
        // First mode activated — register the handler
        signals::register_signal(Signal::TimerTick.as_u8(), timer_tick_handler);
        dbg_println!("timer: signal handler registered");
    } else if old_modes != 0 && new_modes == 0 {
        // Last mode deactivated — unregister and clear the display
        signals::unregister_signal(Signal::TimerTick.as_u8());
        // Clear the entire status region so no stale text remains
        vga_clear_region(STATUS_ROW, 58, VGA_BUFFER_WIDTH, current_color);
        dbg_println!("timer: signal handler unregistered");
    }
}

fn set_mode(mode_bit: u8, enabled: bool) {
    let current_color: ColorCode = ColorCode::new(get_current_colors().0, get_current_colors().1);
    let old = ACTIVE_MODES.load(Ordering::Relaxed);
    let new = if enabled {
        old | mode_bit
    } else {
        old & !mode_bit
    };

    ACTIVE_MODES.store(new, Ordering::Relaxed);
    update_signal_registration(old, new);
    let current_color: ColorCode = ColorCode::new(get_current_colors().0, get_current_colors().1);
    // If disabling a specific mode, clear its region
    if !enabled {
        match mode_bit {
            MODE_COUNTER => vga_clear_region(STATUS_ROW, 58, 68, current_color),
            MODE_UPTIME => vga_clear_region(STATUS_ROW, 70, 78, current_color),
            MODE_HEARTBEAT => vga_clear_region(STATUS_ROW, 79, 80, current_color),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Enable/disable the raw tick counter display (top-right, 10 chars).
pub fn set_counter(enabled: bool) {
    set_mode(MODE_COUNTER, enabled);
    let state = if enabled { "ON" } else { "OFF" };
    println!("Timer counter: {}", state);
}

/// Enable/disable the HH:MM:SS uptime display.
pub fn set_uptime(enabled: bool) {
    set_mode(MODE_UPTIME, enabled);
    let state = if enabled { "ON" } else { "OFF" };
    println!("Timer uptime: {}", state);
}

/// Enable/disable the heartbeat indicator (single pulsing char).
pub fn set_heartbeat(enabled: bool) {
    set_mode(MODE_HEARTBEAT, enabled);
    let state = if enabled { "ON" } else { "OFF" };
    println!("Timer heartbeat: {}", state);
}

/// Master on/off — enables or disables ALL display modes at once.
pub fn enable() {
    let old = ACTIVE_MODES.load(Ordering::Relaxed);
    let new = MODE_COUNTER | MODE_UPTIME | MODE_HEARTBEAT;
    ACTIVE_MODES.store(new, Ordering::Relaxed);
    update_signal_registration(old, new);
    println!(
        "Timer: all modes ON (ticks: {})",
        TICK_COUNT.load(Ordering::Relaxed)
    );
}

pub fn disable() {
    let old = ACTIVE_MODES.load(Ordering::Relaxed);
    ACTIVE_MODES.store(0, Ordering::Relaxed);
    update_signal_registration(old, 0);
    println!(
        "Timer: all modes OFF (ticks frozen at: {})",
        TICK_COUNT.load(Ordering::Relaxed)
    );
}

/// Return the current tick count.
pub fn get_ticks() -> u32 {
    TICK_COUNT.load(Ordering::Relaxed)
}

/// Return whether any timer display mode is currently active.
pub fn is_enabled() -> bool {
    ACTIVE_MODES.load(Ordering::Relaxed) != 0
}

/// Print current status.
pub fn print_status() {
    let modes = ACTIVE_MODES.load(Ordering::Relaxed);
    let ticks = TICK_COUNT.load(Ordering::Relaxed);
    let secs = ticks / PIT_HZ;
    println!("Timer status:");
    println!("  Ticks:     {}", ticks);
    println!("  Uptime:    {}s", secs);
    println!(
        "  Counter:   {}",
        if modes & MODE_COUNTER != 0 {
            "ON"
        } else {
            "OFF"
        }
    );
    println!(
        "  Uptime:    {}",
        if modes & MODE_UPTIME != 0 {
            "ON"
        } else {
            "OFF"
        }
    );
    println!(
        "  Heartbeat: {}",
        if modes & MODE_HEARTBEAT != 0 {
            "ON"
        } else {
            "OFF"
        }
    );
}

// ---------------------------------------------------------------------------
// Shell command handler
//
// Usage:
//   timer on         — enable all modes
//   timer off        — disable all modes
//   timer counter    — toggle counter display
//   timer uptime     — toggle uptime display
//   timer beat       — toggle heartbeat
//   timer status     — print current state
// ---------------------------------------------------------------------------

pub fn shell_command(args: &[&str]) {
    match args.first() {
        Some(&"on") => enable(),
        Some(&"off") => disable(),
        Some(&"counter") => set_counter(ACTIVE_MODES.load(Ordering::Relaxed) & MODE_COUNTER == 0),
        Some(&"uptime") => set_uptime(ACTIVE_MODES.load(Ordering::Relaxed) & MODE_UPTIME == 0),
        Some(&"beat") => set_heartbeat(ACTIVE_MODES.load(Ordering::Relaxed) & MODE_HEARTBEAT == 0),
        Some(&"status") => print_status(),
        _ => println!("Usage: timer on|off|counter|uptime|beat|status"),
    }
}
