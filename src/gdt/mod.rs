//https://wiki.osdev.org/Global_Descriptor_Table#Table
//https://wiki.osdev.org/GDT_Tutorial#Basics
use rlibc::memcpy;
mod descriptor;
use descriptor::SegmentDescriptor;
mod tss;
use tss::TssSegment;
pub const GDTADDR: usize = 0x00000800;
const GDTSIZE: usize = 8;

extern "C" {
	fn gdtflush(_gdtr : *const GdtDescriptor);
	fn tssflush();
}
//WARNING : This is not portable for future x64
#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
struct GdtDescriptor {
	size : u16,
	address : usize
}

impl GdtDescriptor {
	pub fn current() -> GdtDescriptor {
		let gdtr = GdtDescriptor::default();
		unsafe { core::arch::asm!("sgdt [{}]", in(reg) &gdtr as *const _);}
		// print!("GDT is {:#?}", gdtr);
        gdtr
	}
}


//https://wiki.osdev.org/GDT_Tutorial#Basics
//https://wiki.osdev.org/Global_Descriptor_Table
pub fn init() {
	
	let mut tss = TssSegment::default();
	let tss_base : u32 = &tss as *const TssSegment as u32;
	let tss_limit : u32 = tss_base + size_of::<TssSegment>() as u32;
	tss.ss0 = 0x18;
	tss.esp0 = 0;
	tss.cs = 0x08 | 0x3;
	tss.gs = 0x10 | 0x3;
	tss.fs = tss.gs; 
	tss.ss = tss.gs;
	tss.ds = tss.gs;
	tss.es = tss.gs;
	
	let segments : [SegmentDescriptor; GDTSIZE] = [
	SegmentDescriptor::new(0, 0, 0, 0), //Null segment 0x0
	SegmentDescriptor::new(0, 0x000FFFF, 0x9A, 0xC), //Kernel Code 0x8
	SegmentDescriptor::new(0, 0x000FFFF, 0x92, 0xC), //Kernel Data 0x10
	SegmentDescriptor::new(0, 0, 0x96, 0xC), //Kernel Stack 0x18
	SegmentDescriptor::new(0, 0xFFFF, 0xFA, 0xC), //User code 0x20
	SegmentDescriptor::new(0, 0xFFFF, 0xF2, 0xC), //User data 0x28
	SegmentDescriptor::new(0, 0, 0xF6, 0xC), //User stack 0x30
	SegmentDescriptor::new(tss_base, tss_limit, 0xE9, 0x0) //Tss Segment 0x38
	];
	// for i in 0..GDTSIZE {
	// 	println!("{}",segments[i]);
	// }
	let gdtr : GdtDescriptor = GdtDescriptor {
		size : (size_of::<SegmentDescriptor>() * segments.len() - 1) as u16,
		address : GDTADDR 
	};
	
	unsafe {
		memcpy(gdtr.address as *mut u8, segments.as_ptr() as *const u8,  segments.len() * size_of::<SegmentDescriptor>() as usize);
		gdtflush(&gdtr as *const GdtDescriptor);
		tssflush();
	}
	println!("GDT load OK");
}

pub fn print() {
	let gdtr = GdtDescriptor::current();
	for i in 0..GDTSIZE {
		let mut gdtdescriptor: SegmentDescriptor = Default::default();
		unsafe {
			memcpy((&mut gdtdescriptor as *mut _) as *mut u8, 
			(gdtr.address + ((size_of::<SegmentDescriptor>() as usize) * i)) as *const u8, //This + 2 is weird, investigate
		8);
		}
		println!("{}",gdtdescriptor);
	}
}