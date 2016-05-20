use std::path::PathBuf;
use img_reader::LabelType;
use ans::{Ans, SplitOffset};
use ans::return_type::{ReturnType, ImgFormat};

use image;


pub struct AnsPathBuilder {
    img_dir: Option<PathBuf>,
    label_type: Option<LabelType>,

    split_size: Option<(u32, u32)>,
    split_offset: (Option<SplitOffset>, Option<SplitOffset>),
    img_format: Option<ImgFormat>,
}

impl AnsPathBuilder {
    pub fn new() -> AnsPathBuilder {
        AnsPathBuilder {
            img_dir: None,
            label_type: None,
            split_size: None,
            split_offset: (None, None),
            img_format: None,
        }
    }
    pub fn set_img_dir(mut self, path: PathBuf) -> AnsPathBuilder {
        self.img_dir = Some(path);
        self
    }

    pub fn set_label_type(mut self, label_type: LabelType) -> AnsPathBuilder {
        self.label_type = Some(label_type);
        self
    }
    pub fn set_split_size(mut self, size: Option<(u32, u32)>) -> AnsPathBuilder {
        self.split_size = size;
        self
    }

    pub fn set_split_offset(mut self,
                            mut offset: (Option<SplitOffset>, Option<SplitOffset>))
                            -> AnsPathBuilder {
        if let (Some(so1), Some(so2)) = offset.clone() {
            if so1.get_value() == 0 || so2.get_value() == 0 {
                offset = (None, None);
            }
        }
        self.split_offset = offset;
        self
    }
    pub fn set_img_type(mut self, format: image::ImageFormat) -> AnsPathBuilder {
        self.img_format = Some(ImgFormat::Img(format));
        self
    }

    pub fn build(self) -> Ans {
        Ans {
            img_dir: self.img_dir.expect("Called AnsPathBuilder.build() without setting img_dir"),
            label_type: self.label_type
                            .expect("Called AnsPathBuilder.build() without setting label_type"),
            split_size: self.split_size,
            split_offset: self.split_offset,
            img_format: self.img_format.unwrap(),
            discard_barrier: None,
        }
    }
}
