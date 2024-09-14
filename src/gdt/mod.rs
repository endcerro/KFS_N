//https://wiki.osdev.org/Global_Descriptor_Table#Table
//https://wiki.osdev.org/GDT_Tutorial#Basics
pub mod define;
mod descriptor;
mod tss;

use core::ptr::addr_of;

use define::*;
use descriptor::SegmentDescriptor;
use tss::TssSegment;

use crate::utils::memcpy;


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
	#[cfg(feature = "verbose")]
	println!("Tss structure initialized...");

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
	#[cfg(feature = "verbose")]
	println!("GDT Segments initialized...");

	// for i in 0..GDTSIZE {
	// 	println!("{}",segments[i]);
	// }
	let gdtr : GdtDescriptor = GdtDescriptor {
		size : (size_of::<SegmentDescriptor>() * segments.len() - 1) as u16,
		address : GDTADDR
	};
	unsafe {
		memcpy(gdtr.address as *mut u8, segments.as_ptr() as *const u8,  segments.len() * size_of::<SegmentDescriptor>() as usize);
		#[cfg(feature = "verbose")]
		println!("GDT copied to 0x{:x}, flushing..", GDTADDR);
		gdtflush(&gdtr as *const GdtDescriptor);
		#[cfg(feature = "verbose")]
		println!("GDT flush OK, TSS..");
		tssflush();
		#[cfg(feature = "verbose")]
		println!("TSS flush OK");
	}
	#[cfg(feature = "gdt_test")]
	gdt_test_suite();
	#[cfg(feature = "verbose")]
	println!("GDT load OK !");
}

pub fn print() {
	let gdtr = GdtDescriptor::current();
	for i in 0..GDTSIZE {
		let mut gdtdescriptor: SegmentDescriptor = Default::default();
		memcpy((&mut gdtdescriptor as *mut _) as *mut u8, 
			(gdtr.address + ((size_of::<SegmentDescriptor>() as usize) * i)) as *const u8,
			8);
		println!("{}",gdtdescriptor);
	}
}
#[cfg(feature = "gdt_test")]
fn verify_gdt_load_structure() {

	let tss_addr = core::ptr::addr_of!(tss::TSS) as *const TssSegment as u32;
	let tss_limit = tss_addr + size_of::<TssSegment>() as u32 - 1;


	let correct_segments : [SegmentDescriptor; GDTSIZE] = [
		SegmentDescriptor::new(0, 0, 0, 0), //Null segment 0x0
		SegmentDescriptor::new(0, 0xFFFF, 0x9A, 0xCF), //Kernel Code 0x8
		SegmentDescriptor::new(0, 0xFFFF, 0x92, 0xCF), //Kernel Data 0x10
		SegmentDescriptor::new(0, 0xFFFF, 0x96, 0xCF), //Kernel Stack 0x18
		SegmentDescriptor::new(0, 0xFFFF, 0xFA, 0xCF), //User code 0x20
		SegmentDescriptor::new(0, 0xFFFF, 0xF2, 0xCF), //User data 0x28
		SegmentDescriptor::new(0, 0xFFFF, 0xF6, 0xCF), //User stack 0x30
		SegmentDescriptor::new(tss_addr, tss_limit, 0xE9, 0x0) //Tss Segment 0x38
	];
	
	let gdtr = GdtDescriptor::current();
	let test_segments : [SegmentDescriptor; GDTSIZE] = [SegmentDescriptor::default(); GDTSIZE];
	memcpy(addr_of!(test_segments) as *mut _, gdtr.address as *const u8, gdtr.size as usize);
	for i in 0..GDTSIZE {
		assert_eq!(correct_segments[i], test_segments[i]);
	}
    println!("GDT loaded and retrived successfully!");
}

#[cfg(feature = "gdt_test")]
pub fn verify_segment_registers() {
    let mut cs: u16;
    let mut ds: u16;
    let mut ss: u16;
    let mut es: u16;
    let mut fs: u16;
    let mut gs: u16;

    unsafe {
        core::arch::asm!("mov {:x}, cs", out(reg) cs);
        core::arch::asm!("mov {:x}, ds", out(reg) ds);
        core::arch::asm!("mov {:x}, ss", out(reg) ss);
        core::arch::asm!("mov {:x}, es", out(reg) es);
        core::arch::asm!("mov {:x}, fs", out(reg) fs);
        core::arch::asm!("mov {:x}, gs", out(reg) gs);
    }

    println!("CS: {:#x}, DS: {:#x}, SS: {:#x}, ES: {:#x}, FS: {:#x}, GS: {:#x}", 
             cs, ds, ss, es, fs, gs);

    // Verify against expected values
    assert_eq!(cs, KERNEL_CODE_SELECTOR, "CS not set correctly");
    assert_eq!(ds, KERNEL_DATA_SELECTOR, "DS not set correctly");
    assert_eq!(ss, KERNEL_STACK_SELECTOR, "SS not set correctly");
    assert_eq!(es, 0x20, "ES not set correctly"); 
    assert_eq!(fs, 0x28, "FS not set correctly");
    assert_eq!(gs, 0x30, "GS not set correctly");

	let cpl: u16;
	unsafe {
    	core::arch::asm!("mov {:x}, cs", out(reg) cpl);
	}
	assert_eq!(cpl & 0x3, 0, "Not running in ring 0 as expected");

    println!("Segment registers verified successfully!");
}

#[cfg(feature = "gdt_test")]
pub fn verify_tss() {
    unsafe {
        let loaded_tr: u16;
        core::arch::asm!("str {:x}", out(reg) loaded_tr);
        
        println!("Loaded TR: {:#x}", loaded_tr);
        assert_eq!(loaded_tr, 0x38, "TSS not loaded correctly");

        println!("TSS base: {:#x}, limit: {:#x}", tss::TSS.esp0, tss::TSS.ss0);
        assert_eq!(tss::TSS.ss0, 0x10, "TSS SS0 not set correctly");
        assert_eq!(tss::TSS.esp0, &stack_top as *const _ as u32, "TSS ESP0 not set correctly");

        println!("TSS verified successfully!");
    }
}


#[cfg(feature = "gdt_test")]
fn gdt_test_suite() {
	verify_gdt_load_structure();
	verify_segment_registers();
	verify_tss();
}
