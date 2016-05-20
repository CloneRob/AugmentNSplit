use image::{Rgb, ImageBuffer};
use ans::Ans;

#[derive(Debug, Clone)]
pub enum Label {
    Sick,
    Healthy,
}

impl Label {
    pub fn determine_label(label_image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
                           color: [u8; 3])
                           -> Label {

        //let set_percentage = 0.2;
        let major_color = Ans::majority_color(&label_image);
        if let Some(mj) = major_color {
            if mj.0 == color {
                Label::Sick
            } else {
                Label::Healthy
            }
        } else {
            Label::Healthy
        }
    }
}
