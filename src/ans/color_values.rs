
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ColorValues {
    RGB([u8; 3]),
    LUMA([u8; 1]),
}
impl ColorValues {
    pub fn compare(&self, c: [u8; 1]) -> bool {
        if let ColorValues::LUMA(ref cv) = *self {
            if c[0] == cv[0] {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
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
        ColorValues::rgb([255,255,255])
    }

    pub fn black_luma() -> ColorValues {
        ColorValues::luma([0])
    }
    pub fn white_luma() -> ColorValues {
        ColorValues::luma([255])
    }
}
