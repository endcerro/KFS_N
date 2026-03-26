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
    Cascade = 34, // Connected to the second PIC
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
    Syscall = 128, // Commonly used for system calls
}

impl Interrupt {
    // Helper method to get the interrupt number
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Option<Interrupt> {
        match value {
            0 => Some(Interrupt::DivideError),
            1 => Some(Interrupt::Debug),
            2 => Some(Interrupt::NonMaskableInterrupt),
            3 => Some(Interrupt::Breakpoint),
            4 => Some(Interrupt::Overflow),
            5 => Some(Interrupt::BoundRangeExceeded),
            6 => Some(Interrupt::InvalidOpcode),
            7 => Some(Interrupt::DeviceNotAvailable),
            8 => Some(Interrupt::DoubleFault),
            9 => Some(Interrupt::CoprocessorSegmentOverrun),
            10 => Some(Interrupt::InvalidTSS),
            11 => Some(Interrupt::SegmentNotPresent),
            12 => Some(Interrupt::StackSegmentFault),
            13 => Some(Interrupt::GeneralProtectionFault),
            14 => Some(Interrupt::PageFault),
            // 15 reserved
            16 => Some(Interrupt::X87FloatingPointException),
            17 => Some(Interrupt::AlignmentCheck),
            18 => Some(Interrupt::MachineCheck),
            19 => Some(Interrupt::SIMDFloatingPointException),
            20 => Some(Interrupt::VirtualizationException),
            21 => Some(Interrupt::ControlProtectionException),
            // 22-31 reserved
            32 => Some(Interrupt::ProgrammableInterruptTimer),
            33 => Some(Interrupt::Keyboard),
            34 => Some(Interrupt::Cascade),
            35 => Some(Interrupt::COM2),
            36 => Some(Interrupt::COM1),
            37 => Some(Interrupt::LPT2),
            38 => Some(Interrupt::FloppyDisk),
            39 => Some(Interrupt::LPT1),
            40 => Some(Interrupt::RealTimeClock),
            41 => Some(Interrupt::ACPI),
            42 => Some(Interrupt::Peripheral1),
            43 => Some(Interrupt::Peripheral2),
            44 => Some(Interrupt::PS2Mouse),
            45 => Some(Interrupt::FPU),
            46 => Some(Interrupt::PrimaryATA),
            47 => Some(Interrupt::SecondaryATA),
            128 => Some(Interrupt::Syscall),
            _ => None,
        }
    }
}

// Additional constants
pub const IRQ0: u8 = 32; // Base IRQ, can be used to calculate others
pub const MAX_INTERRUPT: u8 = 255;
pub const USER_DEFINED_INTERRUPTS_START: u8 = 48;
pub const USER_DEFINED_INTERRUPTS_END: u8 = 255;
