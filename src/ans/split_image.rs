use image::{ImageBuffer, RgbImage, Rgb};
use ans::label::Label;

#[derive(Clone)]
pub struct SplitImage {
    source: String,
    pub image: Option<RgbImage>,
    pub label: Option<Label>,
    dimension: (u32, u32),
    x_offset: u32,
    y_offset: u32,
}

impl SplitImage {
    pub fn new(src: String,
               img: RgbImage,
               label: Label,
               dim: (u32, u32),
               x: u32,
               y: u32)
               -> SplitImage {
        SplitImage {
            source: src,
            image: Some(img),
            label: Some(label),
            dimension: dim,
            x_offset: x,
            y_offset: y,
        }
    }

    pub fn build(src: String,
               x_dim: u32,
               y_dim: u32,
               x: u32,
               y: u32)
               -> SplitImage {
        SplitImage {
            source: src,
            image: None,
            label: None,
            dimension: (x_dim, y_dim),
            x_offset: x,
            y_offset: y,
        }
    }

    pub fn print(&self) {
        println!("{}\nx: {}, y: {}\n", self.source, self.x_offset, self.y_offset);
    }

    pub fn get_name(&self) -> &str {
        &self.source
    }

    pub fn get_x_dim(&self) -> u32 {
        self.dimension.0
    }

    pub fn get_y_dim(&self) -> u32 {
        self.dimension.1
    }

    pub fn get_x_offset(&self) -> u32 {
        self.x_offset
    }

    pub fn get_y_offset(&self) -> u32 {
        self.y_offset
    }

    pub fn get_image(&self) -> &Option<RgbImage> {
        &self.image
    }

    pub fn set_image(&mut self, img: ImageBuffer<Rgb<u8>, Vec<u8>>) {
        self.image = Some(img);
    }
}
