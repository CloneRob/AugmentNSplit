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
use ans::color_values::ColorValues;
use ans::label::Label;
use ans::color_values;
use std::option::IntoIter;

static SEED:[usize; 4] = [1, 3, 3, 7];


fn main() {
    let train_path = PathBuf::from("/media/data/Projects/barrett/Bilder/images");
    let label_path = PathBuf::from("/media/data/Projects/barrett/Bilder/mask_and");

    let label_type = LabelType::Img(label_path);

    let mut augment_split = AugmentSplitBuilder::new()
        .set_img_dir(train_path)
        .set_label_type(label_type)
        .set_split_size(Some((224u32, 224u32)))
        .set_split_offset((Some(SplitOffset::Val(190u32)), Some(SplitOffset::Val(190u32))))
        .set_img_type(ImageFormat::PNG)
        .with_rotation()
        .set_output_real("data/3Jul/train/real")
        .set_output_mask("data/3Jul/train/mask")
        .build();


    let mut img_reader = ImgReader::new(augment_split.get_imgdir(), augment_split.get_label_type());
    let cv = color_values::ColorValues::white_luma();

    let rng = rand::thread_rng();
    let img_tuple = img_reader.img_map.get("pat24_im1_ACHD");

    if let Some(entry) = img_tuple {
        //let os = Oversamp::new(&entry.0, &entry.1, &rng, cv);

    }

    /*


    let mut s = Split { ratio: None };

    augment_split.split(&mut img_reader, cv, &mut s);

    let mut os = Oversample { ratio: None };
    augment_split.oversample(&mut img_reader, 0.0004, cv, &mut os);
    */
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

struct Oversamp<'rng> {
    curr_position: (u32, u32),
    real: image::DynamicImage,
    mask: image::DynamicImage,
    rng: &'rng rand::ThreadRng,
    candidates: Option<Vec<(u32, u32)>>,
    color_val: ColorValues,

}

impl<'rng> Oversamp<'rng> { 
    fn new(real: image::DynamicImage, mask: image::DynamicImage,
           rng: &'rng rand::ThreadRng, cv: ColorValues) -> Oversamp<'rng> {
        let os = Oversamp {
            curr_position: (0, 0),
            real: real,
            mask: mask,
            rng: rng,
            candidates: None,
            color_val: cv,
        };
        os.extract_pixel()

         
    }

    fn extract_pixel(mut self) -> Oversamp<'rng> {
        let candidates = if let DynamicImage::ImageLuma8(ref mask) = self.mask {
            let whitepx = mask.enumerate_pixels()
                .filter(|x| self.color_val.compare(x.2.data))
                .map(|x| (x.0, x.1))
                .collect::<Vec<_>>();
            Some(whitepx)
        } else {
            None
        };
        self.candidates = candidates;
        self
    } 

    fn next(self) -> IntoIter<Vec<(u32, u32)>> {
        self.candidates.into_iter()

    }
}



