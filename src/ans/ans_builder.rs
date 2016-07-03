use std::path::PathBuf;
use img_reader::LabelType;
use ans::{augment_split, SplitOffset};
use ans::return_type::ImgFormat;

use image;


pub struct AugmentSplitBuilder {
    img_dir: Option<PathBuf>,
    label_type: Option<LabelType>,

    split_size: Option<(u32, u32)>,
    split_offset: (Option<SplitOffset>, Option<SplitOffset>),
    img_format: Option<ImgFormat>,
    rotation: bool,
    output_real: Option<PathBuf>,
    output_mask: Option<PathBuf>,
}

impl AugmentSplitBuilder {
    pub fn new() -> AugmentSplitBuilder {
        AugmentSplitBuilder {
            img_dir: None,
            label_type: None,
            split_size: None,
            split_offset: (None, None),
            img_format: None,
            rotation: false,
            output_real: None,
            output_mask: None,
        }
    }
    pub fn set_img_dir(mut self, path: PathBuf) -> AugmentSplitBuilder {
        self.img_dir = Some(path);
        self
    }
    pub fn set_output_real(mut self, path: &str) -> AugmentSplitBuilder {
        self.output_real = Some(PathBuf::from(path));
        self
    }
    pub fn set_output_mask(mut self, path: &str) -> AugmentSplitBuilder {
        self.output_mask = Some(PathBuf::from(path));
        self
    }

    pub fn set_label_type(mut self, label_type: LabelType) -> AugmentSplitBuilder {
        self.label_type = Some(label_type);
        self
    }
    pub fn set_split_size(mut self, size: Option<(u32, u32)>) -> AugmentSplitBuilder {
        self.split_size = size;
        self
    }
    pub fn with_rotation(mut self) -> AugmentSplitBuilder {
        self.rotation = true;
        self
    }

    pub fn set_split_offset(mut self,
                            mut offset: (Option<SplitOffset>, Option<SplitOffset>))
                            -> AugmentSplitBuilder {
        if let (Some(so1), Some(so2)) = offset.clone() {
            if so1.get_value() == 0 || so2.get_value() == 0 {
                offset = (None, None);
            }
        }
        self.split_offset = offset;
        self
    }
    pub fn set_img_type(mut self, format: image::ImageFormat) -> AugmentSplitBuilder {
        self.img_format = Some(ImgFormat::Img(format));
        self
    }

    pub fn build(self) -> augment_split::AugmentSplit {
        augment_split::AugmentSplit::build(self.img_dir
                                               .expect("Called AugmentSplitBuilder.build() \
                                                        without setting img_dir"),
                                           self.label_type
                                               .expect("Called AugmentSplitBuilder.build() \
                                                        without setting label_type"),
                                           self.split_size,
                                           self.split_offset,
                                           self.img_format.unwrap(),
                                           None,
                                           self.rotation,
                                           self.output_real.unwrap(),
                                           self.output_mask)
    }
}
