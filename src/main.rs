#![allow(dead_code)]
#![feature(step_by)]

extern crate image;
extern crate rand;

mod ans;
mod img_reader;

// use std::fs::File;
use std::path::PathBuf;
// use std::collections::HashMap;
//use time::PreciseTime;

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
    let training_path = PathBuf::from("/media/robert/Lokaler \
                                       Datenträger/BachelorArbeit/Bilder/subset");
    let label_path = PathBuf::from("/media/robert/Lokaler \
                                    Datenträger/BachelorArbeit/Bilder/subset_mask");

    let label_type = LabelType::Img(label_path);

    //let now = PreciseTime::now();
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


    //let finish = PreciseTime::now();
   // let duration = now.to(finish);
    //println!("{:?} ns to create Ans struct", duration.num_nanoseconds());

    //let now = PreciseTime::now();
    let mut img_reader = ImgReader::new(augment_split.get_imgdir(), augment_split.get_label_type());
    //let finish = PreciseTime::now();
    //let duration = now.to(finish);
    //println!("{:?} ms to create img_reader", duration.num_milliseconds());



    //let now = PreciseTime::now();
    let mut s = Split { ratio: None };
    let cv = color_values::ColorValues::white_luma();

    augment_split.split(&mut img_reader, cv, &mut s);

    let mut os = Oversample { ratio: None };
    augment_split.oversample(&mut img_reader, 0.0004, cv, &mut os);
    //let finish = PreciseTime::now();
    //let duration = now.to(finish);
    //println!("{:?} ms to split images", duration.num_milliseconds());
}
