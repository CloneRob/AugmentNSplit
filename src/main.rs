
#![allow(dead_code)]
#![allow(unused_imports)]
extern crate image;
extern crate xml;

mod ans;
mod img_reader;

use std::fs::File;
use std::path::Path;

use img_reader::{ImgReader, LabelType};

fn main() {
    /*
    let mut file = File::create("batch.bin").unwrap();
    let test_img = image::open(&Path::new("pat01_im1_NDBT.tif"));

    let config_path = Path::new("/home/robert/Projects/rust/AugmentNSplit/test_config.xml");

    let training_path = Path::new("/home/robert/Projects/ba/images");

    let label_path = Path::new("/home/robert/Projects/ba/mask_diff");
    let label_type = LabelType::Img(label_path);

    let img_reader = ImgReader::new(training_path, label_type);
    let test_ans = ans::Ans::from_config(config_path);
    */
}
