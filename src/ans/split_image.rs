use image::DynamicImage;
use ans::label::Label;
use std::mem;
use rand::*;

#[derive(Clone)]
pub struct SplitImage {
    source: String,
    pub real: Option<DynamicImage>,
    pub mask: Option<DynamicImage>,
    pub label: Option<Label>,
    dimension: (u32, u32),
    pub rotation: u8,
    x_offset: u32,
    y_offset: u32,
}

impl SplitImage {
    pub fn new(src: &String,
               real: DynamicImage,
               mask: DynamicImage,
               label: Label,
               dim: (u32, u32),
               rot: u8,
               x: u32,
               y: u32)
               -> SplitImage {
        SplitImage {
            source: src.clone(),
            real: Some(real),
            mask: Some(mask),
            label: Some(label),
            dimension: dim,
            rotation: rot,
            x_offset: x,
            y_offset: y,
        }
    }

    pub fn build(src: &String, x_dim: u32, y_dim: u32, rot: u8, x: u32, y: u32) -> SplitImage {
        SplitImage {
            source: src.clone(),
            real: None,
            mask: None,
            label: None,
            dimension: (x_dim, y_dim),
            rotation: rot,
            x_offset: x,
            y_offset: y,
        }
    }

    pub fn print(&self) {
        println!("{}\nx: {}, y: {}\n",
                 self.source,
                 self.x_offset,
                 self.y_offset);
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

    pub fn get_real(&self) -> &Option<DynamicImage> {
        &self.real
    }
    pub fn get_mask(&self) -> &Option<DynamicImage> {
        &self.mask
    }

    pub fn set_real(&mut self, img: DynamicImage) {
        self.real = Some(img);
    }
    pub fn set_mask(&mut self, img: DynamicImage) {
        self.mask = Some(img);
    }

    pub fn get_rotation(&self) -> u8 {
        self.rotation
    }

    pub fn set_rotation(&mut self, r: u8) {
        self.rotation = r;
    }

    pub fn random_rotation(mut self, rng: &mut StdRng) -> Option<SplitImage> {
        if rng.gen_range(0, 100) < 40 {
            self.rotation = rng.gen_range(0, 4);

            if self.rotation != 0 {
                let real = mem::replace(&mut self.real, None);
                let mask = mem::replace(&mut self.mask, None);

                if let (Some(mut real_), Some(mut mask_)) = (real, mask) {
                    match self.rotation {
                        1 => {
                            real_ = real_.rotate90();
                            mask_ = mask_.rotate90();
                        }
                        2 => {
                            real_ = real_.rotate180();
                            mask_ = mask_.rotate180();
                        }
                        3 => {
                            real_ = real_.rotate270();
                            mask_ = mask_.rotate270();
                        }
                        _ => {}
                    };
                    self.set_real(real_);
                    self.set_mask(mask_);
                }
                Some(self)
            } else {
                None
            }
        } else {
            None
        }
    }
}
