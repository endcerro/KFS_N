pub const IDT_SIZE              : usize = 256;

pub const DPL0_INTERRUPT_GATE   : u8 = 0x8E;
pub const DPL3_INTERRUPT_GATE   : u8 = 0xEE;
pub const DPL0_TRAP_GATE        : u8 = 0x8F;
pub const DPL3_TRAP_GATE        : u8 = 0xEF;
pub const DPL0_TASK_GATE        : u8 = 0x85;
pub const DPL3_TASK_GATE        : u8 = 0xE5;