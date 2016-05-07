#![allow(dead_code)]
#![allow(unused_imports)]
extern crate image;
extern crate xml;
extern crate rand;

mod ans;
mod img_reader;

use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::ffi::OsString;

use img_reader::{ImgReader, LabelType};

use image::*;
use ans::{Ans, AnsPathBuilder, SplitOffset};

fn main() {

    //let config_path = PathBuf::from("/home/robert/Projects/rust/AugmentNSplit/test_config.xml");

    let training_path = PathBuf::from("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/image_subset");
    let label_path = PathBuf::from("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/mask_subset");

    //let training_path = PathBuf::from("/home/robert/Projects/ba/images");
    //let label_path = PathBuf::from("/home/robert/Projects/ba/mask_and");

    let label_type = LabelType::Img(label_path);

    let mut ans = AnsPathBuilder::new().set_img_dir(training_path).set_label_type(label_type).set_split_size(Some((224u32, 224u32))).set_split_offset((Some(SplitOffset::Val(100u32)), Some(SplitOffset::Val(100u32)))).set_batches(1000).build();


    let splitimage_vec  = ans.fill_split_vec();
    println!("{:?} Images in splitvec", splitimage_vec.len());
    ans.convert_vec_to_binary(splitimage_vec);

}
