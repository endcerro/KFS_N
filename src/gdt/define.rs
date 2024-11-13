pub const KERNEL_CODE_SELECTOR  : u16   = 0x08;
pub const KERNEL_DATA_SELECTOR  : u16   = 0x10;
pub const KERNEL_STACK_SELECTOR : u16   = 0x18;
#[allow(dead_code)]
pub const USER_CODE_SELECTOR    : u16   = 0x20 | 0x3;
#[allow(dead_code)]
pub const USER_DATA_SELECTOR    : u16   = 0x28 | 0x3;
#[allow(dead_code)]
pub const USER_STACK_SELECTOR   : u16   = 0x30 | 0x3;
#[allow(dead_code)]
pub const TSS_SELECTOR          : u16   = 0x38;
pub const KERNEL_VIRTUAL_BASE   : u32   = 0xC0000000;
pub const GDTADDR               : usize = 0xC0000800; // Was 0x00000800
pub const GDTSIZE               : usize = 8;