// shell/commands/vread.rs
//
// Shell command: vread <addr> [u8|u32|u64]
//
// Reads a value from virtual address `addr` and prints it.
// Optional second argument selects read width (default: u32).
// Address correctness is the caller's responsibility —
// reading from an unmapped address will trigger a page fault.
//
// addr accepts hex (0x...) or decimal.
//
// Examples:
//   vread 0xD0000000
//   vread 0xD0000000 u8
//   vread 0xD0000000 u64

pub fn run(args: &[&str]) {
    if args.is_empty() || args.len() > 2 {
        println!("\nUsage: vread <addr> [u8|u32|u64]");
        println!("  addr   virtual address to read from (hex or decimal)");
        println!("  width  u8 | u32 | u64               (default: u32)");
        println!("\nNote: reading from an unmapped address will page fault.");
        println!("\nExamples:");
        println!("  vread 0xD0000000");
        println!("  vread 0xD0000000 u8");
        println!("  vread 0xD0000000 u64");
        return;
    }

    let addr: u32 = match parse_u32(args[0]) {
        Some(v) => v,
        None    => { println!("\nvread: invalid address '{}'", args[0]); return; }
    };

    let width = if args.len() == 2 { args[1] } else { "u32" };

    println!("\nvread({:#010x}, {})...", addr, width);

    match width {
        "u8" => {
            let val = unsafe { (addr as *const u8).read_volatile() };
            println!("  [{:#010x}] = {:#04x}  ({})", addr, val, val);
        }
        "u32" => {
            let val = unsafe { (addr as *const u32).read_volatile() };
            println!("  [{:#010x}] = {:#010x}  ({})", addr, val, val);
        }
        "u64" => {
            let val = unsafe { (addr as *const u64).read_volatile() };
            println!("  [{:#010x}] = {:#018x}  ({})", addr, val, val);
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