use super::define::*;

//https://www.youtube.com/watch?v=UikDD0VYiME
//https://www.sandpile.org/x86/tss.htm
//https://www.sandpile.org/x86/tss.htm
#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed(4))]
pub struct TssSegment {
	pub link : u16, link_high : u16,
	pub esp0 : u32,
	pub ss0 : u16, 	ss0_high : u16,
	pub esp1 : u32,
	pub ss1 : u16, ss1_high : u16,
	pub esp2 : u32,
	pub ss2 : u16, ss2_high : u16,
	pub cr3 : u32,
	pub eip : u32,
	pub eflags : u32,
	pub eax : u32,
	pub ecx : u32,
	pub edx : u32,
	pub ebx : u32,
	pub esp : u32,
	pub ebp : u32,
	pub esi : u32,
	pub edi : u32,
	pub es : u16, es_high :u16,
	pub cs : u16, cs_high :u16,
	pub ss : u16, ss_high :u16,
	pub ds : u16, ds_high :u16,
	pub fs : u16, fs_high :u16,
	pub gs : u16, gs_high :u16,
	pub ldtr : u16, ldtr_high :u16,
	iopb_low : u16, pub iopb :u16,
	pub ssp : u32
}

impl TssSegment {
	const fn new() -> Self {
		TssSegment {
			link : 0, link_high : 0,
			esp0 : 0,
			ss0 : 0, 	ss0_high : 0,
			esp1 : 0,
			ss1 : 0, ss1_high : 0,
			esp2 : 0,
			ss2 : 0, ss2_high : 0,
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
			es : 0, es_high :0,
			cs : 0, cs_high :0,
			ss : 0, ss_high :0,
			ds : 0, ds_high :0,
			fs : 0, fs_high :0,
			gs : 0, gs_high :0,
			ldtr : 0, ldtr_high :0,
			iopb_low : 0, iopb :0,
			ssp : 0
		}
	}
	pub fn init(&mut self, kernel_stack: u32) -> Result<(),&'static str>
	{
		if kernel_stack == 0 {
			print!("The address is {}", kernel_stack);
			return Err("Invalid kernel stack address");
		}
		//Stack pointer and segment selector
		self.esp0 = kernel_stack;
		self.ss0 = KERNEL_DATA_SELECTOR;

		//Other selectotrs

		self.cs = KERNEL_CODE_SELECTOR;
		self.ss = KERNEL_STACK_SELECTOR;
		self.ds = KERNEL_DATA_SELECTOR;
		self.es = KERNEL_DATA_SELECTOR;
		self.fs = KERNEL_DATA_SELECTOR;
		self.gs = KERNEL_DATA_SELECTOR;

		self.iopb = size_of::<TssSegment>() as u16;
		self.cr3 = 0; //This will change when paging is on
		self.eflags = 0x2;

		return Ok(());
	}
	pub fn _set_usermode_selector(&mut self)
	{
		self.cs = USER_CODE_SELECTOR;
		self.ss = USER_STACK_SELECTOR;
		self.ds = USER_DATA_SELECTOR;
		self.es = USER_DATA_SELECTOR;
		self.fs = USER_DATA_SELECTOR;
		self.gs = USER_DATA_SELECTOR;
	}
	pub fn _set_kernelmode_selector(&mut self)
	{
		self.cs = KERNEL_CODE_SELECTOR;
		self.ss = KERNEL_STACK_SELECTOR;
		self.ds = KERNEL_DATA_SELECTOR;
		self.es = KERNEL_DATA_SELECTOR;
		self.fs = KERNEL_DATA_SELECTOR;
		self.gs = KERNEL_DATA_SELECTOR;
	}
}


pub static mut TSS : TssSegment = TssSegment::new();