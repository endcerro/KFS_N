// shell/commands/vmalloc.rs
//
// Shell command: vmalloc <addr> <size>
//
// Maps `size` bytes of virtual address space starting at `addr`.
// Both addr and size are rounded up to the nearest page boundary.
// Any address is accepted; the only hard limit is the recursive-mapping
// region at 0xFFC00000+ which is a hardware constraint.
//
// Both arguments accept hex (0x...) or decimal.
//
// Examples:
//   vmalloc 0x1000 4096
//   vmalloc 0xD0001234 0x3000   — addr rounded up to 0xD0002000

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
        println!("\nUsage: vmalloc <addr> <size>");
        println!("  addr  virtual address (hex or decimal, rounded up to page boundary)");
        println!("  size  bytes to map   (rounded up to page boundary)");
        println!("\nExample: vmalloc 0xD0000000 4096");
        return;
    }

    let addr = match parse_u32(args[0]) {
        Some(v) => v,
        None => {
            println!("\nvmalloc: invalid address '{}'", args[0]);
            return;
        }
    };
    let size = match parse_usize(args[1]) {
        Some(v) => v,
        None => {
            println!("\nvmalloc: invalid size '{}'", args[1]);
            return;
        }
    };

    println!("\nvmalloc({:#010x}, {} bytes)...", addr, size);

    match vmm::vmalloc(addr, size) {
        Ok((aligned, pages)) => {
            let mapped = pages * 4096;
            if aligned != addr {
                println!(
                    "  Note: address rounded up {:#010x} -> {:#010x}",
                    addr, aligned
                );
            }
            println!("  OK - {} page(s) mapped ({} bytes)", pages, mapped);
            println!(
                "  Range: {:#010x} .. {:#010x}",
                aligned,
                aligned as usize + mapped
            );
        }
        Err(vmm::VmError::RecursiveRegion) => {
            println!("  Error: range overlaps the recursive page-directory region (0xFFC00000+).")
        }
        Err(vmm::VmError::AlreadyMapped) => {
            println!("  Error: one or more pages already mapped. Run vfree first.")
        }
        Err(vmm::VmError::OutOfMemory) => println!("  Error: no free physical frames available."),
        Err(vmm::VmError::ZeroSize) => println!("  Error: size must be > 0."),
    }
}
