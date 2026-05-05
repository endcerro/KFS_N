// shell/commands/vwrite.rs
//
// Shell command: vwrite <addr> <value> [u8|u32|u64]
//
// Writes `value` to virtual address `addr` (rounded up to page boundary).
// Optional third argument selects write width (default: u32).
// Address correctness is the caller's responsibility -
// writing to an unmapped address will trigger a page fault.
//
// Examples:
//   vwrite 0xD0000000 0xDEADBEEF
//   vwrite 0xD0000000 255 u8
//   vwrite 0xD0000000 0xCAFEBABEDEADBEEF u64

use super::parse::{parse_u32, parse_u64};

pub fn run(args: &[&str]) {
    if args.len() < 2 || args.len() > 3 {
        println!("\nUsage: vwrite <addr> <value> [u8|u32|u64]");
        println!(
            "  addr   virtual address to write to (hex or decimal, rounded up to page boundary)"
        );
        println!("  value  value to write               (hex or decimal)");
        println!("  width  u8 | u32 | u64               (default: u32)");
        println!("\nNote: writing to an unmapped address will page fault.");
        println!("\nExamples:");
        println!("  vwrite 0xD0000000 0xDEADBEEF");
        println!("  vwrite 0xD0000000 42 u8");
        println!("  vwrite 0xD0000000 0xCAFEBABEDEADBEEF u64");
        return;
    }

    let addr = match parse_u32(args[0]) {
        Some(v) => v,
        None => {
            println!("\nvwrite: invalid address '{}'", args[0]);
            return;
        }
    };

    // let aligned = page_align_up(addr);
    let aligned = addr;
    if aligned != addr {
        println!(
            "\nNote: address rounded up {:#010x} -> {:#010x}",
            addr, aligned
        );
    }

    let width = if args.len() == 3 { args[2] } else { "u32" };

    println!("\nvwrite({:#010x}, {}, {})...", aligned, args[1], width);

    match width {
        "u8" => {
            let val = match parse_u64(args[1]).and_then(|v| u8::try_from(v).ok()) {
                Some(v) => v,
                None => {
                    println!("  Error: '{}' is not a valid u8 (0-255).", args[1]);
                    return;
                }
            };
            unsafe {
                (aligned as *mut u8).write_volatile(val);
            }
            println!("  Wrote u8  {:#04x} ({}) to [{:#010x}]", val, val, aligned);
        }
        "u32" => {
            let val = match parse_u32(args[1]) {
                Some(v) => v,
                None => {
                    println!("  Error: '{}' is not a valid u32.", args[1]);
                    return;
                }
            };
            unsafe {
                (aligned as *mut u32).write_volatile(val);
            }

            println!("  Wrote u32 {:#010x} ({}) to [{:#010x}]", val, val, aligned);
        }
        "u64" => {
            let val = match parse_u64(args[1]) {
                Some(v) => v,
                None => {
                    println!("  Error: '{}' is not a valid u64.", args[1]);
                    return;
                }
            };
            unsafe {
                (aligned as *mut u64).write_volatile(val);
            }
            println!("  Wrote u64 {:#018x} to [{:#010x}]", val, aligned);
        }
        other => {
            println!("  Error: unknown width '{}'. Use u8, u32, or u64.", other);
        }
    }
}
