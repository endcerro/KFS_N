//https://wiki.osdev.org/Global_Descriptor_Table#Table
//https://wiki.osdev.org/GDT_Tutorial#Basics
mod define;
mod descriptor;
mod tss;

use core::ptr::addr_of;

use define::*;
use descriptor::SegmentDescriptor;
use tss::TssSegment;

use rlibc::memcpy;


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
	//Retrieve the current GDT
	pub fn current() -> GdtDescriptor {
		let gdtr = GdtDescriptor::default();
		unsafe { core::arch::asm!("sgdt [{}]", in(reg) &gdtr as *const _);}
		// print!("GDT is {:#?}", gdtr);
        gdtr
	}
}
extern "C" {
    static stack_top: u8;
}

//https://wiki.osdev.org/GDT_Tutorial#Basics
//https://wiki.osdev.org/Global_Descriptor_Table
pub fn init() {
	let tss_addr : u32;
	let tss_limit : u32;
	unsafe {
		tss::TSS.init(addr_of!(stack_top) as u32).expect("Unvalid TSS stack address");
		tss_addr = core::ptr::addr_of!(tss::TSS) as *const TssSegment as u32;
		tss_limit = tss_addr + size_of::<TssSegment>() as u32 - 1;
	}

	let segments : [SegmentDescriptor; GDTSIZE] = [
		SegmentDescriptor::new(0, 0, 0, 0), //Null segment 0x0
		SegmentDescriptor::new(0, 0xFFFF, 0x9A, 0xCF), //Kernel Code 0x8
		SegmentDescriptor::new(0, 0xFFFF, 0x92, 0xCF), //Kernel Data 0x10
		SegmentDescriptor::new(0, 0xFFFF, 0x96, 0xCF), //Kernel Stack 0x18
		SegmentDescriptor::new(0, 0xFFFF, 0xFA, 0xCF), //User code 0x20
		SegmentDescriptor::new(0, 0xFFFF, 0xF2, 0xCF), //User data 0x28
		SegmentDescriptor::new(0, 0xFFFF, 0xF6, 0xCF), //User stack 0x30
		SegmentDescriptor::new(tss_addr, tss_limit, 0xE9, 0x0) //Tss Segment 0x38
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