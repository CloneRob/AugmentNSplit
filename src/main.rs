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

use rand::*;
use image::*;
use ans::SplitOffset;
use ans::ans_builder::AugmentSplitBuilder;
use ans::augment_split::FindLabel;
use ans::color_values::ColorValues;
use ans::label::Label;
use ans::color_values;

static SEED:[usize; 4] = [1, 3, 3, 7];


fn main() {
    let train_path = PathBuf::from("/media/rcbe-titan/Daten/Projects/barrett/Bilder/images");
    let label_path = PathBuf::from("/media/rcbe-titan/Daten/Projects/barrett/Bilder/mask_and");

    let label_type = LabelType::Img(label_path);

    let mut augment_split = AugmentSplitBuilder::new()
        .set_img_dir(train_path)
        .set_label_type(label_type)
        .set_split_size(Some((50u32, 50u32)))
        .set_split_offset((Some(SplitOffset::Val(42u32)), Some(SplitOffset::Val(42u32))))
        .set_img_type(ImageFormat::PNG)
        .with_rotation()
        .set_output_real("data/train/real")
        .set_output_mask("data/train/mask")
        .build();


    let mut img_reader = ImgReader::new(augment_split.get_imgdir(), augment_split.get_label_type());
    println!("ImgReader constructed");
    let cv = color_values::ColorValues::white_luma();


    /*
    let rng = rand::thread_rng();
    let img_tuple = img_reader.img_map.get("pat24_im1_ACHD");

    if let Some(entry) = img_tuple {
        //let os = Oversamp::new(&entry.0, &entry.1, &rng, cv);
    }

    */

    //let mut s = Split { ratio: None };
    //augment_split.split(&mut img_reader, cv, &mut s);
    //println!("Barrett constructed");
    let mut os = Oversample { ratio: None };
    augment_split.oversample(&mut img_reader, 0.4, cv, &mut os);
    //println!("Cancer constructed");
}


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


struct OversampleIter {
    rng: rand::ThreadRng,
    candidates: Vec<(u32, u32)>,
}

impl OversampleIter { 
    fn new(mask: image::DynamicImage, 
           mut rng: rand::ThreadRng,
           sample_mpy: f32,
           cv: ColorValues) -> Result<OversampleIter, &'static str> {

        let pixels = OversampleIter::extract_pixel(&mask, cv);

        if let Some(px) = pixels {
            let sample_size = (sample_mpy * px.len() as f32) as usize;
            let candidates = rand::sample(&mut rng, px, sample_size);
            let os = OversampleIter {
                rng: rng,
                candidates: candidates,
            };
            Ok(os)
        } else {
            Err("Could not extrat pixels from mask")
        }
    }

    fn extract_pixel(mask: &image::DynamicImage, cv: ColorValues) -> Option<Vec<(u32, u32)>> {
        if let DynamicImage::ImageLuma8(ref mask) = *mask {
            let whitepx = mask.enumerate_pixels()
                .filter(|x| cv.compare(x.2.data))
                .map(|x| (x.0, x.1))
                .collect::<Vec<_>>();
            Some(whitepx)
        } else {
            None
        }
    } 
}
