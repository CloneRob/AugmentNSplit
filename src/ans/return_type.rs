use image::*;

pub struct ReturnType {
    layout: ImgLayout,
    format: ImgFormat,
}

impl ReturnType {
    pub fn get_format(&self) -> &ImgFormat {
        &self.format
    }

    pub fn get_layout(&self) -> &ImgLayout {
        &self.layout
    }
}
pub enum ImgLayout {
    ColorChannel,
    HumanReadable,
}

pub enum ImgFormat {
    Binary {
        batch_size: usize,
    },
    Img(ImageFormat),
}
