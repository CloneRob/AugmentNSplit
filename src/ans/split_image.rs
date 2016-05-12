use std::ffi::OsString;
use image::{ImageBuffer, RgbImage, Rgb};
use ans::label::Label;

pub struct SplitImage {
    source: OsString,
    image: RgbImage,
    label: Label,
    dimension: (u32, u32),
    x_offset: u32,
    y_offset: u32,
}

impl SplitImage {
    pub fn new(src: OsString,
               img: RgbImage,
               label: Label,
               dim: (u32, u32),
               x: u32,
               y: u32)
               -> SplitImage {
        SplitImage {
            source: src,
            image: img,
            label: label,
            dimension: dim,
            x_offset: x,
            y_offset: y,
        }
    }

    pub fn get_image(&self) -> &RgbImage {
        &self.image
    }

    pub fn set_image(&mut self, img: ImageBuffer<Rgb<u8>, Vec<u8>>) {
        self.image = img;
    }
}
