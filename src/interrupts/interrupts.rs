#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Interrupt {
    // CPU Exceptions (0-31)
    DivideError = 0,
    Debug = 1,
    NonMaskableInterrupt = 2,
    Breakpoint = 3,
    Overflow = 4,
    BoundRangeExceeded = 5,
    InvalidOpcode = 6,
    DeviceNotAvailable = 7,
    DoubleFault = 8,
    CoprocessorSegmentOverrun = 9,
    InvalidTSS = 10,
    SegmentNotPresent = 11,
    StackSegmentFault = 12,
    GeneralProtectionFault = 13,
    PageFault = 14,
    // 15 is reserved
    X87FloatingPointException = 16,
    AlignmentCheck = 17,
    MachineCheck = 18,
    SIMDFloatingPointException = 19,
    VirtualizationException = 20,
    ControlProtectionException = 21,
    // 22-31 are reserved

    // Hardware Interrupts (32-47)
    // Note: These are default mappings and can be remapped
    ProgrammableInterruptTimer = 32,
    Keyboard = 33,
    Cascade = 34,  // Connected to the second PIC
    COM2 = 35,
    COM1 = 36,
    LPT2 = 37,
    FloppyDisk = 38,
    LPT1 = 39,
    RealTimeClock = 40,
    ACPI = 41,
    Peripheral1 = 42,
    Peripheral2 = 43,
    PS2Mouse = 44,
    FPU = 45,
    PrimaryATA = 46,
    SecondaryATA = 47,

    // Software Interrupts
    Syscall = 128,  // Commonly used for system calls
}

impl Interrupt {
    // Helper method to get the interrupt number
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    // Helper method to create an Interrupt from a number
    pub fn from_u8(value: u8) -> Option<Interrupt> {
        if value <= 47 || value == 0x80 {
            // Safety: We know these values are valid
            Some(unsafe { core::mem::transmute(value) })
        } else {
            None
        }
    }
}

// Additional constants
pub const IRQ0: u8 = 32;  // Base IRQ, can be used to calculate others
pub const MAX_INTERRUPT: u8 = 255;
pub const USER_DEFINED_INTERRUPTS_START: u8 = 48;
pub const USER_DEFINED_INTERRUPTS_END: u8 = 255;