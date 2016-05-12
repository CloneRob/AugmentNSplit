use image::{Rgb, ImageBuffer};
use ans::Ans;

pub enum Label {
    Sick,
    Healthy,
}

impl Label {
    pub fn determine_label(label_image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
                           color: [u8; 3],
                           set_percentage: f32)
                           -> Label {
        let label = if Ans::check_color(&label_image, color, set_percentage) {
            Label::Sick
        } else {
            Label::Healthy
        };
        label
    }
}
