pub struct pageBitMap([u32; 256]);

impl pageBitMap {
    fn getPageStatus(&self, pageNumber: usize) -> u32 {
        if (pageNumber < 8) {
            (self.0[0] >> pageNumber) & 1
        }
        (self.0[pageNumber / 8] >> pageNumber % 8) & 1
    }
}
