use std::fs::File;
use std::io::prelude::*;
use std::ffi::OsString;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;

use rand::{thread_rng, sample};

use xml::reader::{EventReader, XmlEvent, Error};

use img_reader::{ImgReader, LabelType};
use image::*;

mod label;
pub mod return_type;
mod split_image;
pub mod ans_builder;

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

    img_format: ImgFormat,
    // Discribes a color and a percentage (f32 beeing between 0.0 and 1.0), for discarding Images
    // which contain more than x percent of color pixels
    discard_barrier: Option<([u8; 3], f32)>,
}

impl<'a> Ans {
    fn write_to_file(&self, split_image: SplitImage, line_file: &mut String) {
        let path = self.img_dir.clone();
        let mut image_path = path.parent().unwrap().to_path_buf();
        image_path.push(Path::new("training_data/"));
        let name = self.create_name(&split_image);
        image_path.push(Path::new(&name[..]));

        if let Some(image) = split_image.image {
            image.save(&image_path);
        }

        line_file.push_str("/");
        line_file.push_str(&name);
        if let Some(label) = split_image.label {
            match label {
                Label::Sick => line_file.push_str(" 1"),
                Label::Healthy => line_file.push_str(" 0"),
            }
            line_file.push_str("\n");
        }
    }

    fn write_line_file(&self, line_file: String) {
        let path = self.img_dir.clone();
        let mut file_path = path.parent().unwrap().to_path_buf();
        file_path.push(Path::new("training_data/"));

        let mut file = File::create(file_path.join("image_description.txt")).unwrap();
        file.write_all(&line_file.into_bytes()[..]);
    }

    fn create_name(&self, split_image: &SplitImage) -> String {
        let split_name = split_image.get_name();
        let mut name = if let Some(dot_index) = split_name.char_indices().find(|&c| c.1 == '.') {
            let name = String::from(split_name.split_at(dot_index.0).0);
            name
        } else {
            String::new()
        };

        name.push('_');
        name.push_str(&split_image.get_x_offset().to_string());
        name.push('_');
        name.push_str(&split_image.get_y_offset().to_string());

        if let ImgFormat::Img(format) = self.img_format {
            use image::ImageFormat::*;
            match format {
                PNG => name.push_str(".png"),
                JPEG => name.push_str(".jpg"),
                BMP => name.push_str(".bmp"),
                TIFF => name.push_str(".tif"),
                _ => ()
            }
        }
        name
    }

    fn to_color_groups(&self, img_crop: RgbImage) -> RgbImage {
        let split_size = self.get_split_size();
        let buffer_length = (split_size.0 * split_size.1)  as usize;

        let mut red_buffer: Vec<u8> = Vec::with_capacity(buffer_length);
        let mut green_buffer: Vec<u8> = Vec::with_capacity(buffer_length);
        let mut blue_buffer: Vec<u8> = Vec::with_capacity(buffer_length);

        let mut img_buffer: Vec<u8> = Vec::with_capacity(buffer_length * 3 + 1);

        for pixel in img_crop.pixels() {
            // pixel.data contains values for red, green and blue channel
            // of the pixel
            red_buffer.push(pixel.data[0]);
            green_buffer.push(pixel.data[1]);
            blue_buffer.push(pixel.data[2]);
        }

        img_buffer.append(&mut red_buffer);
        img_buffer.append(&mut green_buffer);
        img_buffer.append(&mut blue_buffer);

        ImageBuffer::from_raw(split_size.0, split_size.1, img_buffer).unwrap()
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
                            let split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, x_current, y_current);
                            if let Some(split_image) = self.split_first_pass(split_image, &mut img_tuple) {
                                self.write_to_file(split_image, &mut line_file)
                            }
                            x_current += x_offset;
                        }
                        y_current += y_offset;
                    }
                    let sick = img_tuple.1.enumerate_pixels().filter(|x| x.2.data == [255, 255, 255]).map(|x| (x.0, x.1)).collect::<Vec<_>>();
                    let mut rng = thread_rng();
                    let sick_len = sick.len();
                    let sample = sample(&mut rng, sick, (0.005 * sick_len as f32) as usize);

                    for s in sample {
                        let cropped_image = imageops::crop(&mut img_tuple.0, s.0, s.1, x_len, y_len).to_image();
                        let mut split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, s.0, s.1);
                        let label = Label::determine_label(&cropped_image, [255, 255, 255]);
                        if let Label::Sick = label {
                            split_image.label = Some(label);

                            if let ImgFormat::Binary{..} = self.img_format {
                                split_image.image = Some(self.to_color_groups(cropped_image));
                            } else {
                                split_image.image = Some(cropped_image);
                            }
                            self.write_to_file(split_image, &mut line_file)
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

    pub fn split(&mut self) {
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
                            let split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, x_current, y_current);
                            if let Some(split_image) = self.split_image(split_image, &mut img_tuple) {
                                self.write_to_file(split_image, &mut line_file)
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
                   img_tuple: &mut (ImageBuffer<Rgb<u8>, Vec<u8>>, ImageBuffer<Rgb<u8>, Vec<u8>>))
                   -> Option<SplitImage> {

        let img_crop = imageops::crop(&mut img_tuple.0, split_image.get_x_offset(), split_image.get_y_offset(), split_image.get_x_dim(), split_image.get_y_dim()).to_image();

        let set_percentage: f32 = 0.20;

        if !Ans::check_color(&img_crop, [0, 0, 0], set_percentage) {
            let label_crop = imageops::crop(&mut img_tuple.1, split_image.get_x_offset(), split_image.get_y_offset(), split_image.get_x_dim(), split_image.get_y_dim()).to_image();

            let label = Label::determine_label(&label_crop, [255, 255, 255]);
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

    /*
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
    */
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
