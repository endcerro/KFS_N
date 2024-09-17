use core::fmt;

//https://wiki.osdev.org/Global_Descriptor_Table

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

impl PartialEq for SegmentDescriptor {
    fn eq(&self, other : &Self) -> bool {
        self.low_limit == other.low_limit &&
        self.low_base == other.low_base &&
        self.mid_base == other.mid_base &&
        self.flags_limit == other.flags_limit &&
        self.high_base == other.high_base
    }
}

impl SegmentDescriptor {
	pub fn new(base : u32, limit : u32, access : u8, flags : u8) -> SegmentDescriptor {
		SegmentDescriptor {
			low_limit : (limit & 0xFFFF ) as u16,
			low_base : (base & 0xFFFF) as u16,
			mid_base : ((base & 0xFF0000) >> 16) as u8,
			access,
			flags_limit : ((limit & 0xF0000 ) >> 16 ) as u8 | (flags & 0xf) << 4,
			high_base : ((base & 0xFF000000) >> 24) as u8,
		}
	}
    pub fn _print_bytes(self) {
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
}
impl fmt::Display for SegmentDescriptor { /*TODO Display access and flag with more granularity */
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let base: u32 = self.low_base as u32| (self.mid_base as u32) << 16 | (self.high_base as u32) << 24;
        let limit : u32 = self.low_limit as u32 | (self.flags_limit as u32 & 0xF ) << 16;
        let flags : u32 = self.flags_limit as u32 & 0xF0;
        let access : u8 = self.access;
        write!(f, "Base {:x}, limit {:x}, flags {:x}, access {:x}", base, limit, flags, access)
    }
}