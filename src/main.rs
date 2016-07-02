#![allow(dead_code)]
#![feature(step_by)]

extern crate image;
extern crate xml;
extern crate rand;
extern crate time;

mod ans;
mod img_reader;

//use std::fs::File;
use std::path::PathBuf;
//use std::collections::HashMap;
use time::PreciseTime;

use img_reader::{ImgReader, LabelType};

use image::*;
use ans::SplitOffset;
use ans::ans_builder::AugmentSplitBuilder;


fn main() {
    //let config_path = PathBuf::from("/home/robert/Projects/rust/AugmentNSplit/test_config.xml");

    let training_path = PathBuf::from("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/images");
    let label_path = PathBuf::from("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/mask_and");

    //let training_path = PathBuf::from("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/subset");
    //let label_path = PathBuf::from("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/subset_mask");

    let label_type = LabelType::Img(label_path);

    let now = PreciseTime::now();
    let mut augment_split = AugmentSplitBuilder::new().set_img_dir(training_path)
                                       .set_label_type(label_type)
                                       .set_split_size(Some((224u32, 224u32)))
                                       .set_split_offset((Some(SplitOffset::Val(190u32)), Some(SplitOffset::Val(190u32))))
                                       .set_img_type(ImageFormat::PNG)
                                       .with_rotation()
                                       .set_output_real("data/2Jul/full/real")
                                       .set_output_mask("data/2Jul/full/mask")
                                       .build();


    let finish = PreciseTime::now();
    let duration = now.to(finish);
    println!("{:?} ns to create Ans struct", duration.num_nanoseconds());

    let now = PreciseTime::now();
    let mut img_reader = ImgReader::new(augment_split.get_imgdir(), augment_split.get_label_type());
    let finish = PreciseTime::now();
    let duration = now.to(finish);
    println!("{:?} ms to create img_reader", duration.num_milliseconds());

    let now = PreciseTime::now();
    let hf = augment_split.build_healthy(&mut img_reader);
    let fs = augment_split.build_sick(&mut img_reader);
    let finish = PreciseTime::now();
    let duration = now.to(finish);
    println!("{:?} ms to split images", duration.num_milliseconds());
    println!("#{}: Healthy, #{}: Fuzzy, #{}: Sick", hf.0, hf.1 + fs.0, fs.1);

    /*
    let now = PreciseTime::now();
    augment_split.build_sick(&mut img_reader);
    let finish = PreciseTime::now();
    let duration = now.to(finish);
    println!("{:?} ms to split images", duration.num_milliseconds());
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
