#[derive(Debug)]
pub(crate) struct Instruction {
    pub(crate) val: [u8; 2],
}

impl Instruction {
    pub(crate) fn new(val: [u8; 2]) -> Self {
        Self { val }
    }
    pub fn get_nn(&self) -> u8 {
        self.val[1]
    }
    pub fn get_nnn(&self) -> u16 {
        let mut ab = (self.val[0] & 0x0F) as u16;
        ab = ab.rotate_left(8);
        ab += self.val[1] as u16;
        ab
    }
    pub fn get_first_nibble(&self) -> u8 {
        self.val[0] & 0xF0
    }
    pub fn get_third_nibble(&self) -> u8 {
        (self.val[1] & 0xF0).rotate_right(4)
    }
    pub fn get_second_nibble(&self) -> u8 {
        self.val[0] & 0x0F
    }
}
