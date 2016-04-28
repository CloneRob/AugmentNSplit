
#![allow(dead_code)]
extern crate image;

mod ans;
mod img_buffer;
mod img_reader;

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

fn main() {
    let mut file = File::create("batch.bin").unwrap();
    let test_img = image::open(&Path::new("pat01_im1_NDBT.tif"));

    if let Ok(img) = test_img {
        file.write_all(&img.raw_pixels()[..]).unwrap();
    } else {
        panic!("Could not open img/path");
    }
    let path = Path::new("/home/robert/Projects/ba/images");
}
