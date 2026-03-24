// shell/commands/vsize.rs
//
// Shell command: vsize <addr>
//
// Counts consecutive mapped pages from `addr` (rounded up to page boundary)
// by walking the live page tables. Returns 0 if not mapped.
//
// Examples:
//   vsize 0xD0000000
//   vsize 4096

use crate::memory::vmm;
use super::parse::{parse_u32, page_align_up};

pub fn run(args: &[&str]) {
    if args.len() != 1 {
        println!("\nUsage: vsize <addr>");
        println!("  addr  virtual address to inspect (hex or decimal)");
        println!("\nExample: vsize 0xD0000000");
        return;
    }

    let addr = match parse_u32(args[0]) {
        Some(v) => v,
        None    => { println!("\nvsize: invalid address '{}'", args[0]); return; }
    };

    let aligned = page_align_up(addr);
    if aligned != addr {
        println!("\nNote: address rounded up {:#010x} -> {:#010x}", addr, aligned);
    }

    let bytes = vmm::vsize(addr);

    if bytes == 0 {
        println!("\nvsize({:#010x}): not mapped", aligned);
    } else {
        let pages = bytes / 4096;
        println!("\nvsize({:#010x}): {} bytes ({} page(s))", aligned, bytes, pages);
        println!("  Range: {:#010x} .. {:#010x}", aligned, aligned as usize + bytes);
    }
}