const PRESENT: u32 = 1 << 0;
const READ_WRITE: u32 = 1 << 1;
const USER_SUPERVISOR: u32 = 1 << 2;
const SUPERVISOR: u32 = 0 << 2;
const WRITE_THROUGH: u32 = 1 << 3;
const CACHE_DISABLE: u32 = 1 << 4;
const ACCESSED: u32 = 1 << 5;
const DIRTY: u32 = 1 << 6;
const PAGE_ATTRIBUTE_TABLE_OFF: u32 = 1 << 7;
const GLOBAL: u32 = 1 << 8; //Should not use this

//These point to the physical frames
struct PageTableEntry(u32);
struct PagingDirectory([PageTableEntry; 1024]);
