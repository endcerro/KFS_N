// shell/commands/vwrite.rs
//
// Shell command: vwrite <addr> <value> [u8|u32|u64]
//
// Writes `value` to virtual address `addr`.
// Optional third argument selects write width (default: u32).
// Address correctness is the caller's responsibility —
// writing to an unmapped address will trigger a page fault.
//
// Both addr and value accept hex (0x...) or decimal.
//
// Examples:
//   vwrite 0xD0000000 0xDEADBEEF
//   vwrite 0xD0000000 255 u8
//   vwrite 0xD0000000 0xCAFEBABEDEADBEEF u64

pub fn run(args: &[&str]) {
    if args.len() < 2 || args.len() > 3 {
        println!("\nUsage: vwrite <addr> <value> [u8|u32|u64]");
        println!("  addr   virtual address to write to (hex or decimal)");
        println!("  value  value to write               (hex or decimal)");
        println!("  width  u8 | u32 | u64               (default: u32)");
        println!("\nNote: writing to an unmapped address will page fault.");
        println!("\nExamples:");
        println!("  vwrite 0xD0000000 0xDEADBEEF");
        println!("  vwrite 0xD0000000 42 u8");
        println!("  vwrite 0xD0000000 0xCAFEBABEDEADBEEF u64");
        return;
    }

    let addr: u32 = match parse_u32(args[0]) {
        Some(v) => v,
        None    => { println!("\nvwrite: invalid address '{}'", args[0]); return; }
    };

    let width = if args.len() == 3 { args[2] } else { "u32" };

    println!("\nvwrite({:#010x}, {}, {})...", addr, args[1], width);

    match width {
        "u8" => {
            let val = match parse_u64(args[1]).and_then(|v| u8::try_from(v).ok()) {
                Some(v) => v,
                None    => { println!("  Error: '{}' is not a valid u8 (0-255).", args[1]); return; }
            };
            unsafe { (addr as *mut u8).write_volatile(val); }
            println!("  Wrote u8  {:#04x} ({}) to [{:#010x}]", val, val, addr);
        }
        "u32" => {
            let val = match parse_u32(args[1]) {
                Some(v) => v,
                None    => { println!("  Error: '{}' is not a valid u32.", args[1]); return; }
            };
            unsafe { (addr as *mut u32).write_volatile(val); }
            println!("  Wrote u32 {:#010x} ({}) to [{:#010x}]", val, val, addr);
        }
        "u64" => {
            let val = match parse_u64(args[1]) {
                Some(v) => v,
                None    => { println!("  Error: '{}' is not a valid u64.", args[1]); return; }
            };
            unsafe { (addr as *mut u64).write_volatile(val); }
            println!("  Wrote u64 {:#018x} to [{:#010x}]", val, addr);
        }
        other => {
            println!("  Error: unknown width '{}'. Use u8, u32, or u64.", other);
        }
    }
}

fn parse_u32(s: &str) -> Option<u32> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u32>().ok()
    }
}

fn parse_u64(s: &str) -> Option<u64> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u64>().ok()
    }
}