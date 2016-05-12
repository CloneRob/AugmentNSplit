use std::fs::File;
use std::io::prelude::*;
use std::ffi::OsString;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;

use xml::reader::{EventReader, XmlEvent, Error};

use img_reader::{ImgReader, LabelType};
use image::*;

mod label;
mod return_type;
mod split_image;
mod ans_builder;

use self::label::*;
use self::return_type::*;
use self::split_image::*;
use self::ans_builder::*;


pub struct Ans {
    img_dir: PathBuf,
    label_type: LabelType,

    split_size: Option<(u32, u32)>,
    // offset for x and y values
    split_offset: (Option<SplitOffset>, Option<SplitOffset>),

    // None for batches meaning single files for each split image
    return_type: return_type::ReturnType,

    // Discribes a color and a percentage (f32 beeing between 0.0 and 1.0), for discarding Images
    // which contain more than x percent of color pixels
    discard_barrier: Option<([u8; 3], f32)>,
}

impl<'a> Ans {
    fn write_to_file(&self, img_buffer: &Vec<u8>, batch_cnt: usize) {
        let path = self.img_dir.clone();
        let mut batch_path = path.parent().unwrap().to_path_buf();
        batch_path.push(Path::new(&("training_data/data_batch_".to_string() +
                                    &*batch_cnt.to_string() +
                                    ".bin")[..]));

        let mut file = File::create(batch_path).unwrap();
        file.write_all(&img_buffer[..]);
    }

    fn to_color_groups(&self, split_image: &mut SplitImage) {
        let split_size = self.get_split_size();
        let buffer_length = (split_size.0 * split_size.1)  as usize;

        let mut red_buffer: Vec<u8> = Vec::with_capacity(buffer_length);
        let mut green_buffer: Vec<u8> = Vec::with_capacity(buffer_length);
        let mut blue_buffer: Vec<u8> = Vec::with_capacity(buffer_length);

        let mut img_buffer: Vec<u8> = Vec::with_capacity(buffer_length * 3 + 1);

        for pixel in split_image.get_image().pixels() {
            // pixel.data contains values for red, green and blue channel
            // of the pixel
            red_buffer.push(pixel.data[0]);
            green_buffer.push(pixel.data[1]);
            blue_buffer.push(pixel.data[2]);
        }

        img_buffer.append(&mut red_buffer);
        img_buffer.append(&mut green_buffer);
        img_buffer.append(&mut blue_buffer);

        split_image.set_image(ImageBuffer::from_raw(split_size.0, split_size.1, img_buffer).unwrap())
    }

    pub fn split(&mut self) {
        let mut img_reader = ImgReader::new(self.img_dir.clone(), self.label_type.clone());

        let mut splitimage_vec: Vec<SplitImage> = Vec::new();

        let set_percentage: f32 = 0.20;
        let mut batch_cnt = 1;

        if let Some((x_len, y_len)) = self.split_size {
            if let (Some(x_offset), Some(y_offset)) = self.split_offset.clone() {

                let x_offset = SplitOffset::get_value(&x_offset);
                let y_offset = SplitOffset::get_value(&y_offset);

                for (name, img_tuple) in img_reader.img_map.iter_mut() {
                    let (x_dim, y_dim) = img_tuple.0.dimensions();

                    let mut y_current = 0u32;
                    while y_current <= y_dim - y_len {

                        let mut x_current = 0u32;
                        while x_current <= x_dim - x_len {

                            if let Some(split_img) = self.split_image(&name,
                                                                      img_tuple,
                                                                      x_current,
                                                                      y_current,
                                                                      x_len,
                                                                      y_len,
                                                                      set_percentage) {
                                  batch_cnt = self.push_vec(split_img, &mut splitimage_vec, batch_cnt);
                            }
                            x_current += x_offset;
                        }
                        y_current += y_offset;
                    }
                }
            } else {
                panic!("aborting Ans::fill_split_vec() due to no offset being specified");
            }
        } else {
            for (name, img_tuple) in img_reader.img_map.iter_mut() {
                let label = Label::determine_label(&img_tuple.1, [0, 0, 0], set_percentage);
                let dimension = img_tuple.0.dimensions();

                let split_img = SplitImage::new(name.clone(),
                                                    img_tuple.0.clone(),
                                                    label,
                                                    (dimension.0, dimension.1),
                                                    0u32,
                                                    0u32);
                batch_cnt = self.push_vec(split_img, &mut splitimage_vec, batch_cnt);
            }
        }
    }
    fn push_vec(&self, img: SplitImage, vec: &mut Vec<SplitImage>, batch_cnt: usize) -> usize {
        match *self.return_type.get_format() {
            ImgFormat::Binary{batch_size: b } => {
                if vec.len() < b {
                    vec.push(img);
                    batch_cnt
                } else {
                    vec.clear();
                    vec.push(img);
                    batch_cnt
                }
            },
            ImgFormat::Img(img_format) =>
                batch_cnt
        }
    }


    fn split_image(&self, name: &OsString,
                   img_tuple: &mut (ImageBuffer<Rgb<u8>, Vec<u8>>, ImageBuffer<Rgb<u8>, Vec<u8>>),
                   x_current: u32,
                   y_current: u32,
                   x_len: u32,
                   y_len: u32,
                   set_percentage: f32)
                   -> Option<SplitImage> {

        let split_img = imageops::crop(&mut img_tuple.0, x_current, y_current, x_len, y_len)
                            .to_image();
        let (x_dim, y_dim) = split_img.dimensions();
        if x_dim < x_len || y_dim < y_len {
            println!("{} {}", x_dim, y_dim);
        }
        if !Ans::check_color(&split_img, [0, 0, 0], set_percentage) {
            let split_label = imageops::crop(&mut img_tuple.1, x_current, y_current, x_len, y_len)
                                  .to_image();
            let label = Label::determine_label(&split_label, [0, 0, 0], set_percentage);

            let mut split_img = SplitImage::new(name.clone(),
                                 split_img,
                                 label,
                                 (x_len, y_len),
                                 x_current,
                                 y_current);

            match *self.return_type.get_format() {
                ImgFormat::Binary{..} => self.to_color_groups(&mut split_img),
                _ => (),
            }
            Some(split_img)
        } else {
            None
        }
    }

    pub fn check_color(image: &RgbImage, color: [u8; 3], percentage: f32) -> bool {
        let dim = {
            let (x, y) = image.dimensions();
            (x as f32, y as f32)
        };
        if let Some(majority_color) = Ans::majority_color(image) {
            if majority_color.0 == color &&
                //percentage of color in given image
               (majority_color.1 as f32 / dim.0 * dim.1) >= percentage {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn majority_color(image: &RgbImage) -> Option<([u8; 3], usize)> {
        let mut color_map: HashMap<[u8; 3], usize> = HashMap::new();
        for pixel in image.pixels() {
            let color_cnt = color_map.entry(pixel.data).or_insert(0);
            *color_cnt += 1;
        }
        let majority_color = color_map.drain().max();
        majority_color
    }
    fn set_split_size(&mut self, x: u32, y: u32) {
        self.split_size = Some((x, y));
    }
    fn get_split_size(&self) -> (u32, u32) {
        if let Some((x, y)) = self.split_size {
            (x, y)
        } else {
            (0, 0)
        }
    }
    fn set_split_offset(&mut self, x: Option<u32>, y: Option<u32>) {
        match (x, y) {
            (Some(x_val), Some(y_val)) => {
                self.split_offset = (Some(SplitOffset::Val(x_val)), Some(SplitOffset::Val(y_val)))
            }
            (Some(x_val), None) => {
                self.split_offset = (Some(SplitOffset::Val(x_val)), Some(SplitOffset::Random))
            }
            (None, Some(y_val)) => {
                self.split_offset = (Some(SplitOffset::Random), Some(SplitOffset::Val(y_val)))
            }
            (None, None) => {
                self.split_offset = (Some(SplitOffset::Random), Some(SplitOffset::Random))
            }
        }
    }

    fn parse_img_dir(xml_events: &mut EventReader<BufReader<File>>,
                     path_builder: &mut AnsPathBuilder) {
        while let Ok(xml_event) = xml_events.next() {
            match xml_event {
                XmlEvent::CData(s) => {
                    // path_builder.set_img_dir(PathBuf::from(s));
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == "img_dir" {
                        break;
                    }
                }
                _ => panic!("Unknown branch found in <img_dir>, please check your configuration"),
            }
        }
    }

    fn parse_label_type(xml_events: &mut EventReader<BufReader<File>>,
                        path_builder: &mut AnsPathBuilder) {
        while let Ok(xml_event) = xml_events.next() {
            match xml_event {
                XmlEvent::StartElement { name, .. } => {
                    match &name.local_name[..] {
                        "label_type" => {}
                        "label_dir" => {}
                        _ => unimplemented!(),
                    }
                }
                XmlEvent::EndElement { name, .. } => {
                    if name.local_name == "label" {
                        break;
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    pub fn from_config(config_path: &Path) -> Option<Ans> {
        let file = File::open(config_path).unwrap();
        let file = BufReader::new(file);

        let mut parser = EventReader::new(file);

        let mut ans_path_builder = AnsPathBuilder::new();


        while let Ok(xml_event) = parser.next() {
            match xml_event {
                XmlEvent::StartElement { name, .. } => {
                    match &name.local_name[..] {
                        "img_dir" => Ans::parse_img_dir(&mut parser, &mut ans_path_builder),
                        // "label" =>,
                        // "split" =>,
                        // "augment" =>,
                        //
                        _ => unimplemented!(),
                    }
                }
                _ => print!("test"),
            }
        }
        unimplemented!()
    }
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
