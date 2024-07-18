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
#[derive(Debug)]
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
	const fn new(base : u32, limit : u32, access : u8, flags : u8) -> SegmentDescriptor {
		SegmentDescriptor {
			low_limit : (limit & 0xffff ) as u16,
			low_base : (base & 0xffff) as u16,
			mid_base : (base & 0xff0000 >> 16) as u8,
			access,
			flags_limit : ((limit & 0xf0000 ) >> 16 ) as u8 | (flags & 0xf) << 4,
			high_base : ((base & 0xff000000) >> 24) as u8,
		}
	}
}

//https://wiki.osdev.org/GDT_Tutorial#Basics
//https://wiki.osdev.org/Global_Descriptor_Table
pub fn init() {
	
	const SEGMENTS : [SegmentDescriptor; GDTSIZE] = [
	SegmentDescriptor::new(0, 0, 0, 0), //Null segment 0x0
	SegmentDescriptor::new(0, 0xFFFFF, 0x9A, 0xC), //Kernel Code 0x8
	SegmentDescriptor::new(0, 0xFFFFF, 0x92, 0xC), //Kernel Data 0x10
	SegmentDescriptor::new(0, 0xFFFFF, 0x96, 0xC), //Kernel Stack 0x18
	SegmentDescriptor::new(0, 0xFFFFF, 0xFA, 0xC), //User code 0x20
	SegmentDescriptor::new(0, 0xFFFFF, 0xF2, 0xC), //User data 0x28
	SegmentDescriptor::new(0, 0xFFFFF, 0xF6, 0xC), //User stack 0x30
	];
	let gdtr : GdtDescriptor = GdtDescriptor {
		size : (size_of::<SegmentDescriptor>() * GDTSIZE - 1) as u16,
		address : GDTADDR
	};
	print!("Gdtr : {:#?}", SEGMENTS.len());

	unsafe {
		memcpy(gdtr.address as *mut u8, SEGMENTS.as_ptr() as *const u8, size_of::<SegmentDescriptor>() * GDTSIZE as usize);
		gdtflush(&gdtr as *const GdtDescriptor);
	}
	println!("GDT Success");
}