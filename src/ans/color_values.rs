
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ColorValues {
    RGB([u8; 3]),
    LUMA([u8; 1]),
}
impl ColorValues {
    pub fn rgb(color: [u8; 3]) -> ColorValues {
        ColorValues::RGB(color)
    }
    pub fn luma(color: [u8; 1]) -> ColorValues {
        ColorValues::LUMA(color)
    }
    pub fn black_rgb() -> ColorValues {
        ColorValues::rgb([0,0,0])
    }
    pub fn white_rgb() -> ColorValues {
        ColorValues::rgb([1,1,1])
    }

    pub fn black_luma() -> ColorValues {
        ColorValues::luma([0])
    }
    pub fn white_luma() -> ColorValues {
        ColorValues::luma([1])
    }
}
