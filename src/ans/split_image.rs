use image::{DynamicImage};
use ans::label::Label;

#[derive(Clone)]
pub struct SplitImage {
    source: String,
    pub image: Option<DynamicImage>,
    pub label: Option<Label>,
    dimension: (u32, u32),
    pub rotation: u8,
    x_offset: u32,
    y_offset: u32,
}

impl SplitImage {
    pub fn new(src: String,
               img: DynamicImage,
               label: Label,
               dim: (u32, u32),
               rot: u8,
               x: u32,
               y: u32)
               -> SplitImage {
        SplitImage {
            source: src,
            image: Some(img),
            label: Some(label),
            dimension: dim,
            rotation: rot,
            x_offset: x,
            y_offset: y,
        }
    }

    pub fn build(src: String,
               x_dim: u32,
               y_dim: u32,
               rot: u8,
               x: u32,
               y: u32)
               -> SplitImage {
        SplitImage {
            source: src,
            image: None,
            label: None,
            dimension: (x_dim, y_dim),
            rotation: rot,
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

    pub fn get_image(&self) -> &Option<DynamicImage> {
        &self.image
    }

    pub fn set_image(&mut self, img: DynamicImage) {
        self.image = Some(img);
    }

    pub fn get_rotation(&self) -> u8 {
        self.rotation
    }

    pub fn set_rotation(&mut self, r: u8) {
        self.rotation = r;
    }
}
