//64 Bits so 8 Bytes
#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SegmentDescriptor {
	low_limit : u16,
	low_base : u16,
	mid_base : u8,
	access : u8,
	flags_limit : u8,
	high_base : u8 
}

impl SegmentDescriptor {
	pub fn new(base : u32, limit : u32, access : u8, flags : u8) -> SegmentDescriptor {
		SegmentDescriptor {
			low_limit : (limit & 0xffff ) as u16,
			low_base : (base & 0xffff) as u16,
			mid_base : (base & 0xff0000 >> 16) as u8,
			access,
			flags_limit : ((limit & 0xf0000 ) >> 16 ) as u8 | (flags & 0xf) << 4,
			high_base : ((base & 0xff000000) >> 24) as u8,
		}
	}
    pub fn print_bytes(self) {
        let low_limit : u16 = self.low_limit;
        let low_base : u16 = self.low_base;
        let mid_base : u8 = self.mid_base;
        let access : u8 = self.access;
        let flags_limit : u8 = self.flags_limit;
        let high_base : u8 = self.high_base;
        print!("low_limit : {:b} ", low_limit);
        print!("low_base : {:b} ", low_base);
        print!("mid_base : {:b} ", mid_base);
        print!("access : {:b} ", access);
        print!("flags_limit : {:b} ", flags_limit);
        println!("high_base : {:b} ", high_base);

    }

	// fn reconstruct(segment : SegmentDescriptor) -> SegmentDescriptor 
	// {

	// }
	pub fn print(&self)
	{
		let mut basea : usize = 0;
		// let mut flags : usize;
		// let mut limit : usize;
		// let mut access : usize;
		basea = self.low_base as usize; //| (self.mid_base as u32) << 16 | (self.high_base as u32) << 24; 
		// limit = self.low_limit as usize | ((self.flags_limit & 0xF) as usize) << 16;
        println!("Base {:x}", basea);
	}
}