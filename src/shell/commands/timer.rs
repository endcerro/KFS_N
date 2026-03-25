pub fn run(args: &[&str]) {
    match args.first() {
        Some(&"on")     => crate::timer::enable(),
        Some(&"off")    => crate::timer::disable(),
        Some(&"status") => {
            let state = if crate::timer::is_enabled() { "ON" } else { "OFF" };
            println!("Timer: {}  Ticks: {}", state, crate::timer::get_ticks());
        }
        _ => println!("Usage: timer on|off|status"),
    }
}