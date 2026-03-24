// shell/commands/vfree.rs
//
// Shell command: vfree <addr> <size>
//
// Unmaps the virtual region [addr, addr+size). Both arguments are rounded
// up to page boundaries to match vmalloc's behaviour.
//
// Examples:
//   vfree 0xD0000000 4096
//   vfree 0xD0000000 0x3000

use crate::memory::vmm;
use super::parse::{parse_u32, parse_usize, page_align_up};

pub fn run(args: &[&str]) {
    if args.len() != 2 {
        println!("\nUsage: vfree <addr> <size>");
        println!("  addr  base address used with vmalloc");
        println!("  size  same size used with vmalloc");
        println!("\nExample: vfree 0xD0000000 4096");
        return;
    }

    let addr = match parse_u32(args[0]) {
        Some(v) => v,
        None    => { println!("\nvfree: invalid address '{}'", args[0]); return; }
    };
    let size = match parse_usize(args[1]) {
        Some(v) => v,
        None    => { println!("\nvfree: invalid size '{}'", args[1]); return; }
    };

    let aligned = page_align_up(addr);
    if aligned != addr {
        println!("\nNote: address rounded up {:#010x} -> {:#010x}", addr, aligned);
    }

    println!("\nvfree({:#010x}, {} bytes)...", aligned, size);

    match vmm::vfree(addr, size) {
        Ok(freed) =>
            println!("  OK - {} page(s) freed ({} bytes)", freed, freed * 4096),
        Err(vmm::VmError::ZeroSize) =>
            println!("  Error: size must be > 0."),
        Err(e) =>
            println!("  Error: {:?}", e),
    }
}