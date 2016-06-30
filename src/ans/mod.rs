use std::fs::File;
use std::mem;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::ffi::OsString;
use std::io::BufReader;
use std::fs::DirBuilder;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use rand::{thread_rng, sample};
use rand::distributions::{IndependentSample, Range};

use xml::reader::{EventReader, XmlEvent, Error};

use img_reader::{ImgReader, LabelType};
use image::*;

pub mod label;
pub mod return_type;
pub mod split_image;
pub mod color_values;
pub mod augment_split;
pub mod ans_builder;

use self::label::*;
use self::return_type::*;
use self::split_image::*;
use self::ans_builder::*;
use self::color_values::ColorValues;

enum ImageKind {
    Real,
    Mask,
}
#[derive(Clone)]
pub enum SplitOffset {
    Random,
    Val(u32),
}

impl SplitOffset {
    pub fn get_value(&self) -> u32 {
        match *self {
            // TODO Make this actually random!!
            SplitOffset::Random => 32u32,
            SplitOffset::Val(x) => x,
        }

    }
}
struct SplitData {
    name: String,
    coords: Vec<(u32, u32)>,
}

impl SplitData {
    fn new(split: Vec<&str>) -> SplitData {
        let mut file_name = String::from(split[0]);
        file_name.push('_');
        file_name.push_str(split[1]);
        file_name.push('_');
        file_name.push_str(split[2]);
        file_name.push_str(".tif");

        SplitData {
            name: file_name,
            coords: vec![],
        }
    }
    fn add_data(&mut self, tuple: (u32, u32)) {
        self.coords.push(tuple);
    }
    fn get_name(&self) -> &str {
        &self.name
    }
}
fn create_splitmap(bufreader: BufReader<&File>) -> HashMap<String, SplitData> {
    let mut img_map: HashMap<String, SplitData> = HashMap::new();

    for line in bufreader.lines() {
        let l = line.unwrap();
        let split = l.split(" ").collect::<Vec<_>>();
        let s = split[0].split("_").collect::<Vec<_>>();

        let mut key = String::from(s[0]);
        key.push_str(s[1]);

        let coord_pair = (s[3].parse::<u32>().unwrap(), s[4].parse::<u32>().unwrap());
        match img_map.entry(key) {
            Entry::Occupied(mut o) => {
                let temp = o.get_mut();
                temp.add_data(coord_pair);
            }
            Entry::Vacant(v) => {
                v.insert(SplitData::new(s));
            }
        }
    }
    img_map
}



    /*
    pub fn build_mask_fromfile(&mut self, img_reader: &mut ImgReader, file_path: &Path) {
        let f = File::open(file_path).unwrap();
        let reader = BufReader::new(&f);
        let map = create_splitmap(reader);

        if let Some((x_len, y_len)) = self.split_size {
            for (name, mut img_tuple) in img_reader.img_map.iter_mut() {
                let name = name.clone().into_string().unwrap();
                let split = name.split("_").collect::<Vec<_>>();
                let mut key = String::from("/");
                key.push_str(split[0]);
                key.push_str(split[1]);

                if let Some(coord_vec) = map.get(&key) {
                    for coords in coord_vec.coords.iter() {
                        let mask_crop = img_tuple.1.crop(coords.0, coords.1, x_len, y_len);
                    }
                }
            }
        }
    }
    pub fn build_sick(&mut self, img_reader: &mut ImgReader) {
        if let Some((x_len, y_len)) = self.split_size {

            let pixels = (x_len * y_len) as f32;
            let mut line_file = String::new();

            for (name, mut img_tuple) in img_reader.img_map.iter_mut() {
                let sick_pixel_vec = img_tuple.1.enumerate_pixels()
                                               .filter(|x| x.2.data == [255, 255, 255])
                                               .map(|x| (x.0, x.1))
                                               .collect::<Vec<_>>();

                let sample_size = (0.00065 * sick_pixel_vec.len() as f32) as usize;
                let mut rng = thread_rng();
                let sampled_pixels = sample(&mut rng, sick_pixel_vec, sample_size);

                for s in sampled_pixels {
                    let real_crop = imageops::crop(&mut img_tuple.0, s.0, s.1, x_len, y_len).to_image();
                    let mask_crop = imageops::crop(&mut img_tuple.1, s.0, s.1, x_len, y_len).to_image();

                    let mask_color_info = Ans::get_color([255, 255, 255], &mask_crop);

                    if real_crop.dimensions() == (x_len, y_len) {
                        if mask_color_info.1 / pixels >= 0.70 {
                            let mut split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, 0, s.0, s.1);
                            split_image.label = Some(Label::Sick);
                            split_image.image = Some(real_crop);

                            self.write_to_file(&split_image, &mut line_file, ImageType::Real)
                        }
                    }
                }
            }
            self.write_line_file(line_file)
        }
    }
    */
    /*
    pub fn build_split(&mut self, img_reader: &mut ImgReader) {

        let black_treshhold = 0.25;
        let white_treshhold = 0.8;

        if let Some((x_len, y_len)) = self.split_size {

            let pixels = (x_len * y_len) as f32;

            if let (Some(x_offset), Some(y_offset)) = self.split_offset.clone() {

                let x_offset = SplitOffset::get_value(&x_offset);
                let y_offset = SplitOffset::get_value(&y_offset);

                let mut line_file = String::new();

                for (name, mut img_tuple) in img_reader.img_map.iter_mut() {
                    let real_dim = img_tuple.0.dimensions();
                    for i in (0 .. real_dim.0 - x_len).step_by(x_offset) {
                        for j in (0 .. real_dim.1 - y_len).step_by(y_offset) {
                            let real_crop = imageops::crop(&mut img_tuple.0, i, j, x_len, y_len).to_image();
                            let mask_crop = imageops::crop(&mut img_tuple.1, i, j, x_len, y_len).to_image();

                            let mask_color_info = Ans::get_color([255, 255, 255], &mask_crop);
                            let real_color_info = Ans::get_color([0, 0, 0], &real_crop);


                            if real_color_info.1 / pixels < black_treshhold {
                                if mask_color_info.1 / pixels < 1.0 - white_treshhold {
                                    let mut split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, 0, i, j);
                                    split_image.label = Some(Label::Healthy);
                                    split_image.image = Some(real_crop);

                                    self.write_to_file(&split_image, &mut line_file, ImageType::Real)
                                }
                            }
                        }
                    }
                }
                self.write_line_file(line_file)
            }
        }
    }


    pub fn oversample_split(&mut self) {
        let mut img_reader = ImgReader::new(self.img_dir.clone(), self.label_type.clone());
        if let Some((x_len, y_len)) = self.split_size {
            if let (Some(x_offset), Some(y_offset)) = self.split_offset.clone() {

                let x_offset = SplitOffset::get_value(&x_offset);
                let y_offset = SplitOffset::get_value(&y_offset);

                let mut line_file = String::new();
                for (name, mut img_tuple) in img_reader.img_map.iter_mut() {
                    let (x_dim, y_dim) = img_tuple.0.dimensions();

                    let mut y_current = 0u32;
                    while y_current <= y_dim - y_len {

                        let mut x_current = 0u32;
                        while x_current <= x_dim - x_len {
                            let split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, 0, x_current, y_current);
                            if let Some(split_image) = self.split_first_pass(split_image, &mut img_tuple) {
                                self.write_to_file(&split_image, &mut line_file, ImageType::Real)
                            }
                            x_current += x_offset;
                        }
                        y_current += y_offset;
                    }
                    let sick = img_tuple.1.enumerate_pixels().filter(|x| x.2.data == [255, 255, 255]).map(|x| (x.0, x.1)).collect::<Vec<_>>();
                    let mut rng = thread_rng();
                    let sick_len = sick.len();
                    let sample = sample(&mut rng, sick, (0.0008 * sick_len as f32) as usize);

                    for s in sample {
                        let cropped_image = imageops::crop(&mut img_tuple.0, s.0, s.1, x_len, y_len).to_image();
                        if cropped_image.dimensions() != (x_len, y_len) {
                            continue
                        }
                        let cropped_label = imageops::crop(&mut img_tuple.1, s.0, s.1, x_len, y_len).to_image();
                        let mut split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, 0, s.0, s.1);
                        let label = Label::determine_label(&cropped_label, [255, 255, 255]);
                        if let Label::Sick = label {
                            split_image.label = Some(label);

                            if let ImgFormat::Binary{..} = self.img_format {
                                split_image.image = Some(self.to_color_groups(cropped_image));
                            } else {
                                split_image.image = Some(cropped_image);
                            }
                            self.write_to_file(&split_image, &mut line_file, ImageType::Real)
                        }
                    }
                }
                self.write_line_file(line_file)
            }
        }
    }

    fn split_first_pass(&self, mut split_image: SplitImage,
                   img_tuple: &mut (ImageBuffer<Rgb<u8>, Vec<u8>>, ImageBuffer<Rgb<u8>, Vec<u8>>))
                   -> Option<SplitImage> {

        let img_crop = imageops::crop(&mut img_tuple.0, split_image.get_x_offset(), split_image.get_y_offset(), split_image.get_x_dim(), split_image.get_y_dim()).to_image();

        let set_percentage: f32 = 0.20;

        if !Ans::check_color(&img_crop, [0, 0, 0], set_percentage) {
            let label_crop = imageops::crop(&mut img_tuple.1, split_image.get_x_offset(), split_image.get_y_offset(), split_image.get_x_dim(), split_image.get_y_dim()).to_image();

            let label = Label::determine_label(&label_crop, [255, 255, 255]);
            if let Label::Healthy = label {
                split_image.label = Some(label);

                if let ImgFormat::Binary{..} = self.img_format {
                    split_image.image = Some(self.to_color_groups(img_crop));
                } else {
                    split_image.image = Some(img_crop);
                }
                Some(split_image)
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn new_split(&mut self, img_reader: &mut ImgReader) {

        if let Some((x_len, y_len)) = self.split_size {
            if let (Some(x_offset), Some(y_offset)) = self.split_offset.clone() {

                let x_offset = SplitOffset::get_value(&x_offset);
                let y_offset = SplitOffset::get_value(&y_offset);

                let mut line_file = String::new();
                for (name, mut img_tuple) in img_reader.img_map.iter_mut() {
                    let (x_dim, y_dim) = img_tuple.0.dimensions();

                    let mut y_current = 0u32;
                    while y_current <= y_dim - y_len {

                        let mut x_current = 0u32;
                        while x_current <= x_dim - x_len {
                            let split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, 0, x_current, y_current);
                            if let Some(mut split_image) = self.split_image(split_image, &mut img_tuple, ImageType::Real) {
                                self.write_to_file(&split_image, &mut line_file, ImageType::Real);
                                let mut rotation_cnt = 1;
                                while rotation_cnt <= self.rotation {
                                    let temp_image = mem::replace(&mut split_image.image, Option::None);
                                    if let Some(rgb_image) = temp_image {
                                        let t = imageops::rotate90(&rgb_image);
                                        mem::replace(&mut split_image.image, Some(t));
                                    }
                                    split_image.rotate();
                                    self.write_to_file(&split_image, &mut line_file, ImageType::Real);
                                    rotation_cnt += 1;
                                }
                            }
                            if let Some(_) = self.output_mask {
                                let split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, 0, x_current, y_current);
                                if let Some(mut split_image) = self.split_image(split_image, &mut img_tuple, ImageType::Mask) {
                                    self.write_to_file(&split_image, &mut line_file, ImageType::Mask);
                                    let mut rotation_cnt = 1;
                                    while rotation_cnt <= self.rotation {
                                        let temp_image = mem::replace(&mut split_image.image, Option::None);
                                        if let Some(rgb_image) = temp_image {
                                            let t = imageops::rotate90(&rgb_image);
                                            mem::replace(&mut split_image.image, Some(t));
                                        }
                                        split_image.rotate();
                                        self.write_to_file(&split_image, &mut line_file, ImageType::Mask);
                                        rotation_cnt += 1;
                                    }
                                }
                            }
                            x_current += x_offset;
                        }
                        y_current += y_offset;
                    }
                }
                self.write_line_file(line_file)

            }
        }
    }

    pub fn split(&mut self, img_reader: &mut ImgReader) {

        if let Some((x_len, y_len)) = self.split_size {
            if let (Some(x_offset), Some(y_offset)) = self.split_offset.clone() {

                let x_offset = SplitOffset::get_value(&x_offset);
                let y_offset = SplitOffset::get_value(&y_offset);

                let mut line_file = String::new();
                for (name, mut img_tuple) in img_reader.img_map.iter_mut() {
                    let (x_dim, y_dim) = img_tuple.0.dimensions();

                    let mut y_current = 0u32;
                    while y_current <= y_dim - y_len {

                        let mut x_current = 0u32;
                        while x_current <= x_dim - x_len {
                            let split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, 0, x_current, y_current);
                            if let Some(split_image) = self.split_image(split_image, &mut img_tuple, ImageType::Real) {
                                self.write_to_file(&split_image, &mut line_file, ImageType::Real)
                            }
                            x_current += x_offset;
                        }
                        y_current += y_offset;
                    }
                }
                self.write_line_file(line_file)

            }
        }
    }

    fn split_image(&self, mut split_image: SplitImage,
                   img_tuple: &mut (ImageBuffer<Rgb<u8>, Vec<u8>>, ImageBuffer<Rgb<u8>, Vec<u8>>),
                   image_type: ImageType)
                   -> Option<SplitImage> {

        let mut img_crop = imageops::crop(&mut img_tuple.0, split_image.get_x_offset(), split_image.get_y_offset(), split_image.get_x_dim(), split_image.get_y_dim()).to_image();

        let set_percentage: f32 = 0.20;

        if !Ans::check_color(&img_crop, [0, 0, 0], set_percentage) {
            let label_crop = imageops::crop(&mut img_tuple.1, split_image.get_x_offset(), split_image.get_y_offset(), split_image.get_x_dim(), split_image.get_y_dim()).to_image();

            let label = Label::determine_label(&label_crop, [255, 255, 255]);
            split_image.label = Some(label);

            if let ImageType::Mask = image_type {
                img_crop = label_crop;
            }

            if let ImgFormat::Binary{..} = self.img_format {
                split_image.image = Some(self.to_color_groups(img_crop));
            } else {
                split_image.image = Some(img_crop);
            }

            Some(split_image)
        } else {
            None
        }
    }


*/
