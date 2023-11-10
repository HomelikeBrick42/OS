pub const TA_INTERRUPT_GATE: u8 = 0b1000_1110;
pub const TA_TRAP_GATE: u8 = 0b1000_1111;

#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
pub struct DescriptorEntry {
    pub offset0: u16,
    pub selector: u16,
    pub ist: u8,
    pub type_attr: u8,
    pub offset1: u16,
    pub offset2: u32,
    pub ignore: u32,
}

impl DescriptorEntry {
    pub fn set_offset(&mut self, offset: u64) {
        self.offset0 = (offset & 0x000000000000FFFF) as u16;
        self.offset1 = ((offset & 0x00000000FFFF0000) >> 16) as u16;
        self.offset2 = ((offset & 0xFFFFFFFF00000000) >> 32) as u32;
    }

    pub fn get_offset(&self) -> u64 {
        self.offset0 as u64 | ((self.offset1 as u64) << 16) | ((self.offset2 as u64) << 32)
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Descriptor {
    pub limit: u16,
    pub offset: u64,
}
