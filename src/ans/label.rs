use image::DynamicImage;
use ans::{augment_split, color_values};

#[derive(Debug, Clone)]
pub enum Label {
    Sick,
    Fuzzy,
    Healthy,
}

impl Label {
    pub fn determine_label(label_image: &DynamicImage, color: color_values::ColorValues) -> Label {
        //let set_percentage = 0.2;
        let major_color = augment_split::AugmentSplit::majority_color(&label_image);
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
