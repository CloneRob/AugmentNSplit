#![allow(dead_code)]
#![allow(unused_imports)]
extern crate image;
extern crate xml;

mod ans;
mod img_reader;

use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::ffi::OsString;

use img_reader::{ImgReader, LabelType};

use image::*;
use ans::Ans;

fn main() {

    //let config_path = PathBuf::from("/home/robert/Projects/rust/AugmentNSplit/test_config.xml");

    let training_path = PathBuf::from("/media/robert/2E722FCC722F979B/Bachelor Arbeit/BIlder");

    let label_path = PathBuf::from("/media/robert/2E722FCC722F979B/Bachelor Arbeit/mask_and");
    let label_type = LabelType::Img(label_path);

    let mut img_reader = ImgReader::new(training_path, label_type);
    println!("{} Images in ImgReader Map", img_reader.get_num_of_images());


    let mut splitimage_vec: Vec<SplitImage> = Vec::new();

    let x_offset = 50u32;
    let y_offset = 50u32;

    let set_percentage: f32 = 0.20;


    for (name, img_tuple) in img_reader.img_map.iter_mut() {
        let (x_dim, y_dim)  = img_tuple.0.dimensions();
        let (mut current_x, mut current_y) = (0u32, 0u32);

        while current_x < x_dim && current_y < y_dim {
            let split_img = imageops::crop(&mut img_tuple.0, current_x, current_y, 224u32, 224u32).to_image();

            if !Ans::check_color(&split_img, [0, 0, 0], set_percentage) {

                let split_label = imageops::crop(&mut img_tuple.1, current_x, current_y, 224u32, 224u32).to_image();
                let label = if Ans::check_color(&split_label, [0, 0, 0], set_percentage) {
                    Label::Sick
                } else {
                    Label::Healthy
                };

                splitimage_vec.push(
                    SplitImage::new(
                        name.clone(),
                        split_img,
                        label,
                        (x_dim, y_dim),
                        current_x,
                        current_y,
                    )
                );

            }
            current_x += x_offset;
            current_y += y_offset;
        }
    }
    println!("{:?} Images in splitvec", splitimage_vec.len());

}


pub enum Label {
    Sick,
    Healthy,
}

pub struct SplitImage {
    source: OsString,
    image: RgbImage,
    label: Label,
    dimension: (u32, u32),
    x_offset: u32,
    y_offset: u32,
}

impl SplitImage {
    pub fn new(src: OsString, img: RgbImage, label: Label, dim: (u32, u32), x: u32, y: u32) -> SplitImage {
        SplitImage {
            source: src,
            image: img,
            label: label,
            dimension: dim,
            x_offset: x,
            y_offset: y,
        }
    }
}
