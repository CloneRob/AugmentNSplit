#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(step_by)]

extern crate image;
extern crate xml;
extern crate rand;
extern crate time;

mod ans;
mod img_reader;

use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::ffi::OsString;
use time::{PreciseTime, Duration};

use img_reader::{ImgReader, LabelType};

use image::*;
use ans::{Ans, SplitOffset};
use ans::ans_builder::AnsPathBuilder;


fn main() {

    //let config_path = PathBuf::from("/home/robert/Projects/rust/AugmentNSplit/test_config.xml");

    let training_path = PathBuf::from("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/images");
    let label_path = PathBuf::from("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/mask_and");

    //let training_path = PathBuf::from("/home/robert/Projects/ba/images");
    //let label_path = PathBuf::from("/home/robert/Projects/ba/mask_and");

    let label_type = LabelType::Img(label_path);

    let now = PreciseTime::now();
    let mut ans = AnsPathBuilder::new().set_img_dir(training_path)
                                       .set_label_type(label_type)
                                       .set_split_size(Some((500u32, 500u32)))
                                       .set_split_offset((Some(SplitOffset::Val(500u32)), Some(SplitOffset::Val(500u32))))
                                       .set_img_type(ImageFormat::PNG)
                                       .set_output_real("data/28Jun/real")
                                       .set_output_mask("data/28Jun/mask")
                                       .set_rotation(3)
                                       .build();


    let finish = PreciseTime::now();
    let duration = now.to(finish);
    println!("{:?} ns to create Ans struct", duration.num_nanoseconds());

    let now = PreciseTime::now();
    let mut img_reader = ImgReader::new(ans.get_imgdir(), ans.get_label_type());
    let finish = PreciseTime::now();
    let duration = now.to(finish);
    println!("{:?} ms to create img_reader", duration.num_milliseconds());

    let now = PreciseTime::now();
    ans.new_split(&mut img_reader);
    let finish = PreciseTime::now();
    let duration = now.to(finish);
    println!("{:?} ms to split images", duration.num_milliseconds());
    /*
    let now = PreciseTime::now();
    ans.build_healthy(&mut img_reader);
    let finish = PreciseTime::now();
    let duration = now.to(finish);
    println!("{:?} ms to split Healthy images", duration.num_milliseconds());

    let now = PreciseTime::now();
    ans.build_sick(&mut img_reader);
    let finish = PreciseTime::now();
    let duration = now.to(finish);
    println!("{:?} ms to split Sick images", duration.num_milliseconds());

    let path = Path::new("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/training_data/pat18_im3_ACHD_0_960.png");
    let mut img = open(&path).unwrap().to_rgb();
    let img = imageops::crop(&mut img, 0, 0, 224, 224);

    let color_info = Ans::get_color([0, 0, 0], &img);

    let pixels = {
        let p = img.dimensions();
        p.0 * p.1
    };

    println!("Pixels: {}, black pixels {}, ratio {}", pixels, color_info.1, color_info.1 / pixels as f32);
    */

}
