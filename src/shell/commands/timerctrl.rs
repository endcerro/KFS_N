use crate::timer::{ACTIVE_MODES, MODE_COUNTER, MODE_HEARTBEAT, MODE_UPTIME};
use core::sync::atomic::Ordering;
// pub fn run(args: &[&str]) {
//     match args.first() {
//         Some(&"on") => crate::timer::enable(),
//         Some(&"off") => crate::timer::disable(),
//         Some(&"status") => {
//             let state = if crate::timer::is_enabled() {
//                 "ON"
//             } else {
//                 "OFF"
//             };
//             println!("\nTimer: {}  Ticks: {}", state, crate::timer::get_ticks());
//         }
//         _ => {
//             println!("");
//             println!("\nUsage: timer on|off|status");
//         }
//     }
// }
pub fn run(args: &[&str]) {
    match args.first() {
        Some(&"on") => crate::timer::enable(),
        Some(&"off") => crate::timer::disable(),
        Some(&"counter") => {
            crate::timer::set_counter(ACTIVE_MODES.load(Ordering::Relaxed) & MODE_COUNTER == 0)
        }
        Some(&"uptime") => {
            crate::timer::set_uptime(ACTIVE_MODES.load(Ordering::Relaxed) & MODE_UPTIME == 0)
        }
        Some(&"beat") => {
            crate::timer::set_heartbeat(ACTIVE_MODES.load(Ordering::Relaxed) & MODE_HEARTBEAT == 0)
        }
        Some(&"status") => crate::timer::print_status(),
        _ => println!("\nUsage: timer on|off|counter|uptime|beat|status"),
    }
}
