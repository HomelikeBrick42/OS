#[repr(C, align(8))]
pub struct Guid([u8; 16]);

impl Guid {
    pub const GRAPHICS_OUTPUT_PROTOCOL: Self = Self::new(
        0x9042a9de, 0x23dc, 0x4a38, 0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a,
    );

    pub const fn new(
        a: u32,
        b: u16,
        c: u16,
        d0: u8,
        d1: u8,
        d2: u8,
        d3: u8,
        d4: u8,
        d5: u8,
        d6: u8,
        d7: u8,
    ) -> Self {
        Self([
            (a & 0xFF) as u8,
            ((a >> 8) & 0xFF) as u8,
            ((a >> 16) & 0xFF) as u8,
            ((a >> 24) & 0xFF) as u8,
            (b & 0xFF) as u8,
            ((b >> 8) & 0xFF) as u8,
            (c & 0xFF) as u8,
            ((c >> 8) & 0xFF) as u8,
            d0,
            d1,
            d2,
            d3,
            d4,
            d5,
            d6,
            d7,
        ])
    }
}
