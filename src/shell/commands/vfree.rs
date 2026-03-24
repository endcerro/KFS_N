// shell/commands/vfree.rs
//
// Shell command: vfree <addr> <size>
//
// Unmaps the virtual region [addr, addr+size). Both arguments are rounded
// up to page boundaries to match vmalloc's behaviour.
// Both arguments accept hex (0x...) or decimal.
//
// Examples:
//   vfree 0xD0000000 4096
//   vfree 0xD0000000 0x3000

use crate::memory::vmm;

fn parse_u32(s: &str) -> Option<u32> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u32>().ok()
    }
}

fn parse_usize(s: &str) -> Option<usize> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        usize::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<usize>().ok()
    }
}

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
        None => {
            println!("\nvfree: invalid address '{}'", args[0]);
            return;
        }
    };
    let size = match parse_usize(args[1]) {
        Some(v) => v,
        None => {
            println!("\nvfree: invalid size '{}'", args[1]);
            return;
        }
    };

    println!("\nvfree({:#010x}, {} bytes)...", addr, size);

    match vmm::vfree(addr, size) {
        Ok(freed) => println!("  OK - {} page(s) freed ({} bytes)", freed, freed * 4096),
        Err(vmm::VmError::ZeroSize) => println!("  Error: size must be > 0."),
        Err(e) => println!("  Error: {:?}", e),
    }
}
