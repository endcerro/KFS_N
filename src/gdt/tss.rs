//https://www.youtube.com/watch?v=UikDD0VYiME
//https://www.sandpile.org/x86/tss.htm
#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed)]
pub struct TssSegment {
	link : u16,
	link_high : u16,
	pub esp0 : u32,
	pub ss0 : u16,
	ss0_high : u16,
	esp1 : u32,
	ss1 : u16,
	ss1_high : u16,
	esp2: u32,
	ss2: u16,
	ss2_high : u16,
	cr3 : u32,
	eip : u32,
	eflags : u32,
	eax : u32,
	ecx : u32,
	edx : u32,
	ebx : u32,
	esp : u32,
	ebp : u32,
	esi : u32,
	edi : u32,
	pub es : u16,
	es_high : u16,
	pub cs : u16,
	cs_high : u16,
	pub ss : u16,
	ss_high : u16,
	pub ds : u16,
	ds_high : u16,
	pub fs : u16,
	fs_high : u16,
	pub gs : u16,
	gs_high : u16,
	ldtr: u16,
	ldtr_high : u16,
	iopb_offset_low : u16,
	iopb_offset : u16,
	ssp : u32
}

pub static mut  TSS : TssSegment = TssSegment{	
	link : 0,
	link_high : 0,
	esp0 : 0,
	ss0 : 0,
	ss0_high : 0,
	esp1 : 0,
	ss1 : 0,
	ss1_high : 0,
	esp2: 0,
	ss2: 0,
	ss2_high : 0,
	cr3 : 0,
	eip : 0,
	eflags : 0,
	eax : 0,
	ecx : 0,
	edx : 0,
	ebx : 0,
	esp : 0,
	ebp : 0,
	esi : 0,
	edi : 0,
	es : 0,
	es_high : 0,
	cs : 0,
	cs_high : 0,
	ss : 0,
	ss_high : 0,
	ds : 0,
	ds_high : 0,
	fs : 0,
	fs_high : 0,
	gs : 0,
	gs_high : 0,
	ldtr: 0,
	ldtr_high : 0,
	iopb_offset_low : 0,
	iopb_offset : 0,
	ssp : 0
};