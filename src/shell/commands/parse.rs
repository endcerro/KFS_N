// shell/commands/parse.rs
//
// Shared argument-parsing helpers used by vmalloc, vfree, vsize,
// vwrite, and vread.

/// Parse a u32 from a decimal or 0x-prefixed hex string.
pub fn parse_u32(s: &str) -> Option<u32> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u32>().ok()
    }
}

/// Parse a usize from a decimal or 0x-prefixed hex string.
pub fn parse_usize(s: &str) -> Option<usize> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        usize::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<usize>().ok()
    }
}

/// Parse a u64 from a decimal or 0x-prefixed hex string.
pub fn parse_u64(s: &str) -> Option<u64> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u64>().ok()
    }
}

/// Round `addr` up to the nearest 4 KB page boundary.
/// If already aligned the value is returned unchanged.
///
/// Used by every v-command so that address handling is consistent:
/// the same address passed to vmalloc, vfree, vsize, vwrite, or vread
/// always resolves to the same page-aligned base.
pub fn page_align_up(addr: u32) -> u32 {
    const PAGE_SIZE: u32 = 4096;
    addr.saturating_add(PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}