#![allow(dead_code)]
#![feature(step_by)]

extern crate image;
extern crate xml;
extern crate rand;
extern crate time;

mod ans;
mod img_reader;

// use std::fs::File;
use std::path::PathBuf;
// use std::collections::HashMap;
use time::PreciseTime;

use img_reader::{ImgReader, LabelType};

use image::*;
use ans::SplitOffset;
use ans::ans_builder::AugmentSplitBuilder;
use ans::augment_split::FindLabel;
use ans::label::Label;
use ans::color_values;

struct Split {
    ratio: Option<f32>,
}

impl FindLabel for Split {
    fn label(&mut self, ratio: f32) -> Option<Label> {
        self.ratio = Some(ratio);
        self.label_fn()
    }
    fn label_fn(&self) -> Option<Label> {
        if let Some(ratio) = self.ratio {
            match ratio {
                0.0...0.2 => Some(Label::Healthy),
                0.8...1.0 => Some(Label::Sick),
                _ => None,
            }
        } else {
            None
        }
    }
}

struct Oversample {
    ratio: Option<f32>,
}


impl FindLabel for Oversample {
    fn label(&mut self, ratio: f32) -> Option<Label> {
        self.ratio = Some(ratio);
        self.label_fn()
    }
    fn label_fn(&self) -> Option<Label> {
        if let Some(ratio) = self.ratio {
            match ratio {
                0.0...0.2 => Some(Label::Healthy),
                0.8...1.0 => Some(Label::Sick),
                _ => None,
            }
        } else {
            None
        }
    }
}


fn main() {
    // let config_path = PathBuf::from("/home/robert/Projects/rust/AugmentNSplit/test_config.xml");

    // let training_path = PathBuf::from("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/images");
    // let label_path = PathBuf::from("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/mask_and");

    let training_path = PathBuf::from("/media/robert/Lokaler \
                                       Datenträger/BachelorArbeit/Bilder/subset");
    let label_path = PathBuf::from("/media/robert/Lokaler \
                                    Datenträger/BachelorArbeit/Bilder/subset_mask");

    let label_type = LabelType::Img(label_path);

    let now = PreciseTime::now();
    let mut augment_split = AugmentSplitBuilder::new()
        .set_img_dir(training_path)
        .set_label_type(label_type)
        .set_split_size(Some((224u32, 224u32)))
        .set_split_offset((Some(SplitOffset::Val(190u32)), Some(SplitOffset::Val(190u32))))
        .set_img_type(ImageFormat::PNG)
        .with_rotation()
        .set_output_real("data/3Jul/train/real")
        .set_output_mask("data/3Jul/train/mask")
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
    let mut s = Split { ratio: None };
    augment_split.split(&mut img_reader, &mut s);

    let mut os = Oversample { ratio: None };
    augment_split.oversample(&mut img_reader,
                             0.0004,
                             color_values::ColorValues::white_luma(),
                             &mut os);
    let finish = PreciseTime::now();
    let duration = now.to(finish);
    println!("{:?} ms to split images", duration.num_milliseconds());

    // let now = PreciseTime::now();
    // augment_split.build_sick(&mut img_reader);
    // let finish = PreciseTime::now();
    // let duration = now.to(finish);
    // println!("{:?} ms to split images", duration.num_milliseconds());
    // let now = PreciseTime::now();
    // ans.build_healthy(&mut img_reader);
    // let finish = PreciseTime::now();
    // let duration = now.to(finish);
    // println!("{:?} ms to split Healthy images", duration.num_milliseconds());
    //
    // let now = PreciseTime::now();
    // ans.build_sick(&mut img_reader);
    // let finish = PreciseTime::now();
    // let duration = now.to(finish);
    // println!("{:?} ms to split Sick images", duration.num_milliseconds());
    //
    // let path = Path::new("/media/robert/Lokaler Datenträger/BachelorArbeit/Bilder/training_data/pat18_im3_ACHD_0_960.png");
    // let mut img = open(&path).unwrap().to_rgb();
    // let img = imageops::crop(&mut img, 0, 0, 224, 224);
    //
    // let color_info = Ans::get_color([0, 0, 0], &img);
    //
    // let pixels = {
    // let p = img.dimensions();
    // p.0 * p.1
    // };
    //
    // println!("Pixels: {}, black pixels {}, ratio {}", pixels, color_info.1, color_info.1 / pixels as f32);
    //

}
