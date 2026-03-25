// shell/commands/vread.rs
//
// Shell command: vread <addr> [u8|u32|u64]
//
// Reads a value from virtual address `addr` (rounded up to page boundary).
// Optional second argument selects read width (default: u32).
// Address correctness is the caller's responsibility -
// reading from an unmapped address will trigger a page fault.
//
// Examples:
//   vread 0xD0000000
//   vread 0xD0000000 u8
//   vread 0xD0000000 u64

use super::parse::{page_align_up, parse_u32};

pub fn run(args: &[&str]) {
    if args.is_empty() || args.len() > 2 {
        println!("\nUsage: vread <addr> [u8|u32|u64]");
        println!(
            "  addr   virtual address to read from (hex or decimal, rounded up to page boundary)"
        );
        println!("  width  u8 | u32 | u64               (default: u32)");
        println!("\nNote: reading from an unmapped address will page fault.");
        println!("\nExamples:");
        println!("  vread 0xD0000000");
        println!("  vread 0xD0000000 u8");
        println!("  vread 0xD0000000 u64");
        return;
    }

    let addr = match parse_u32(args[0]) {
        Some(v) => v,
        None => {
            println!("\nvread: invalid address '{}'", args[0]);
            return;
        }
    };

    let aligned = page_align_up(addr);
    if aligned != addr {
        println!(
            "\nNote: address rounded up {:#010x} -> {:#010x}",
            addr, aligned
        );
    }

    let width = if args.len() == 2 { args[1] } else { "u32" };

    println!("\nvread({:#010x}, {})...", aligned, width);

    match width {
        "u8" => {
            let val = unsafe { (aligned as *const u8).read_volatile() };
            println!("  [{:#010x}] = {:#04x}  ({})", aligned, val, val);
        }
        "u32" => {
            let val = unsafe { (aligned as *const u32).read_volatile() };
            println!("  [{:#010x}] = {:#010x}  ({})", aligned, val, val);
        }
        "u64" => {
            let val = unsafe { (aligned as *const u64).read_volatile() };
            println!("  [{:#010x}] = {:#018x}  ({})", aligned, val, val);
        }
        other => {
            println!("  Error: unknown width '{}'. Use u8, u32, or u64.", other);
        }
    }
}
