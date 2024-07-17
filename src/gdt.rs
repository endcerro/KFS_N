//https://wiki.osdev.org/Global_Descriptor_Table#Table
//https://wiki.osdev.org/GDT_Tutorial#Basics

use rlibc::memcpy;

pub const GDTADDR: u32 = 0x00000800;
const GDTSIZE: usize = 7;

extern "C" {
    fn gdtflush(_gdtr : *const GdtDescriptor);
}
//WARNING : This is not portable for future x64
#[repr(C, packed)]
struct GdtDescriptor {
    size : u16,
    address : u32
}
#[repr(C, packed)]
struct SegmentDescriptor {
    low_limit : u16,
    low_base : u16,
    mid_base : u8,
    access : u8,
    flags_limit : u8,
    high_base : u8 
}

impl SegmentDescriptor {
    fn new(base : u32, limit : u32, access : u8, flags : u8) -> SegmentDescriptor {
        SegmentDescriptor {
            low_limit : (limit & 0xffff ) as u16,
            low_base : (base & 0xffff) as u16,
            mid_base : (base & 0xff0000 >> 16) as u8,
            access : access,
            flags_limit : ((limit & 0xf0000 ) >> 16 ) as u8 | (flags & 0xf) << 4,
            high_base : ((base & 0xff000000) >> 24) as u8,
        }
    }
}

//https://wiki.osdev.org/GDT_Tutorial#Basics
//https://wiki.osdev.org/Global_Descriptor_Table
pub fn init() {
    let gdtr : GdtDescriptor = GdtDescriptor {
        size : (size_of::<SegmentDescriptor>() * GDTSIZE) as u16,
        address : GDTADDR 
    };

    let segments : [SegmentDescriptor; GDTSIZE] = [
        SegmentDescriptor::new(0, 0, 0, 0), //Null segment
        SegmentDescriptor::new(0, 0xFFFF, 0x9A, 0xC), //Kernel Code
        SegmentDescriptor::new(0, 0xFFFF, 0x92, 0xC), //Kernel Data
        SegmentDescriptor::new(0, 0, 0x96, 0xC), //Kernel Stack
        SegmentDescriptor::new(0, 0xFFFF, 0xFA, 0xC), //User code
        SegmentDescriptor::new(0, 0xFFFF, 0xF2, 0xC), //User data
        SegmentDescriptor::new(0, 0, 0xF6, 0xC), //User stack
    ];

    unsafe {
        memcpy(gdtr.address as *mut u8, segments.as_ptr() as *const u8, gdtr.size as usize);
        gdtflush(&gdtr as *const GdtDescriptor);
    }
    println!("GDT Success");
}