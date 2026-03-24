// shell/commands/vsize.rs
//
// Shell command: vsize <addr>
//
// Counts consecutive mapped pages from `addr` (rounded up to page boundary)
// by walking the live page tables. Returns 0 if not mapped.
// addr accepts hex (0x...) or decimal.
//
// Examples:
//   vsize 0xD0000000
//   vsize 4096

use crate::memory::vmm;

fn parse_u32(s: &str) -> Option<u32> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u32>().ok()
    }
}

pub fn run(args: &[&str]) {
    if args.len() != 1 {
        println!("\nUsage: vsize <addr>");
        println!("  addr  virtual address to inspect (hex or decimal)");
        println!("\nExample: vsize 0xD0000000");
        return;
    }

    let addr = match parse_u32(args[0]) {
        Some(v) => v,
        None => {
            println!("\nvsize: invalid address '{}'", args[0]);
            return;
        }
    };

    let bytes = vmm::vsize(addr);

    if bytes == 0 {
        println!("\nvsize({:#010x}): not mapped", addr);
    } else {
        let pages = bytes / 4096;
        // Replicate the same rounding vmm::vsize uses so the range is accurate
        let aligned = (addr.saturating_add(0xFFF)) & !0xFFF;
        println!(
            "\nvsize({:#010x}): {} bytes ({} page(s))",
            addr, bytes, pages
        );
        println!(
            "  Range: {:#010x} .. {:#010x}",
            aligned,
            aligned as usize + bytes
        );
    }
}
