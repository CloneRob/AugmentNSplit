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

mod label;
pub mod return_type;
mod split_image;
pub mod ans_builder;

use self::label::*;
use self::return_type::*;
use self::split_image::*;
use self::ans_builder::*;

enum ImageType {
    Real,
    Mask,
}


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

    rotation: u8,

    output_real: PathBuf,
    output_mask: Option<PathBuf>,

}

impl<'a> Ans {
    pub fn get_imgdir(&self) -> PathBuf {
        self.img_dir.clone()
    }

    pub fn get_label_type(&self) -> LabelType {
        self.label_type.clone()
    }
    fn write_to_file(&self, split_image: &SplitImage, line_file: &mut String, image_type: ImageType) {
        let path = self.img_dir.clone();
        let mut image_path = path.parent().unwrap().to_path_buf();
        match image_type {
            ImageType::Real => image_path.push(self.output_real.clone()),
            ImageType::Mask => {
                if let Some(ref out) = self.output_mask {
                    image_path.push(out.clone());
                }
            }
        }
        DirBuilder::new().recursive(true).create(&image_path).unwrap();
        let name = self.create_name(&split_image);
        image_path.push(Path::new(&name[..]));

        if let Some(ref image) = split_image.image {
            match image_type {
                ImageType::Real => {
                    let _ = image.save(&image_path);
                },
                ImageType::Mask => {
                    let dim = self.split_size.unwrap();
                    let mut buffer = ImageBuffer::<Luma<u8>, Vec<u8>>::new(dim.0, dim.1);
                    for (x, y, pixel) in image.enumerate_pixels().filter(|p| p.2.data != [0, 0, 0]) {
                        let mut pixel = pixel.to_luma();
                        pixel.data = [1];
                        buffer.put_pixel(x, y, pixel);
                    };
                    let _ = buffer.save(&image_path);
                }
            }
        }

        if let ImageType::Real = image_type {
            line_file.push_str("/");
            line_file.push_str(&name);
            if let Some(ref label) = split_image.label {
                match *label {
                    Label::Sick => line_file.push_str(" 1"),
                    Label::Healthy => line_file.push_str(" 0"),
                }
                line_file.push_str("\n");
            }
        }
    }

    fn write_line_file(&self, line_file: String) {
        let path = self.img_dir.clone();
        let mut file_path = path.parent().unwrap().to_path_buf();
        file_path.push(self.output_real.clone());

        DirBuilder::new().recursive(true).create(&file_path).unwrap();

        //let mut file = File::create(file_path.join("image_description.txt")).unwrap();
        let mut file = OpenOptions::new()
                                    .write(true)
                                    .append(true)
                                    .create(true)
                                    .open(file_path.join("image_description.txt"))
                                    .unwrap();

        file.write(&line_file.into_bytes()[..]);
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

        let rotation = match split_image.get_rotation(){
            0 => String::from("000deg"),
            1 => String::from("090deg"),
            2 => String::from("180deg"),
            3 => String::from("270deg"),
            _ => String::from("err"),
        };
        name.push('_');
        name.push_str(&rotation);

        if let Some(ref label) = split_image.label {
            match *label {
                Label::Sick => name.push_str("_Sick"),
                Label::Healthy => name.push_str("_Healthy"),
            }
        }

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
                        let mask_crop = imageops::crop(&mut img_tuple.1, coords.0, coords.1, x_len, y_len);
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
    fn random_rotation(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let range = Range::new(1,3);
        let mut rng = thread_rng();

        let rotation = range.ind_sample(&mut rng);

        let temp = imageops::rotate180(image);
        match rotation {
            //1 => image = imageops::rotate90(&image),
            //2 => image = imageops::rotate180(&image),
            //3 => image = imageops::rotate270(&image),
            _ => {}
        };

    }
    pub fn build_healthy(&mut self, img_reader: &mut ImgReader) {
        let black_treshhold = 0.35;
        let white_treshhold = 0.8;

        if let Some((x_len, y_len)) = self.split_size {

            let pixels = (x_len * y_len) as f32;

            if let (Some(x_offset), Some(y_offset)) = self.split_offset.clone() {

                let x_offset = SplitOffset::get_value(&x_offset);
                let y_offset = SplitOffset::get_value(&y_offset);
                println!("x offset: {:?}, y offset: {}", x_offset, y_offset);
                println!("x len: {:?}, y len: {}", x_len, y_len);

                let mut line_file = String::new();

                for (name, mut img_tuple) in img_reader.img_map.iter_mut() {
                    let real_dim = img_tuple.0.dimensions();
                    for i in (0 .. real_dim.0 - x_len + 1).step_by(x_offset) {
                        for j in (0 .. real_dim.1 - y_len + 1).step_by(y_offset) {

                            let real_crop = imageops::crop(&mut img_tuple.0, i, j, x_len, y_len).to_image();
                            let mask_crop = imageops::crop(&mut img_tuple.1, i, j, x_len, y_len).to_image();

                            let real_color_info = Ans::get_color([0, 0, 0], &real_crop);
                            let mask_color_info = Ans::get_color([255, 255, 255], &mask_crop);

                            if real_color_info.1 / pixels < black_treshhold {
                                if mask_color_info.1 / pixels < 1.0 - white_treshhold {
                                    let mut split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, 0, i, j);
                                    split_image.label = Some(Label::Healthy);
                                    split_image.image = Some(real_crop);

                                    self.write_to_file(&split_image, &mut line_file, ImageType::Real)
                                }
                            } else {
                                    println!("thrown out outer" );
                            }
                        }
                    }
                }
                self.write_line_file(line_file)
            }
        }
    }
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

    pub fn get_color(color: [u8; 3], image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> ([u8; 3], f32){
        let color_cnt = image.pixels().filter(|x| x.data == color).count();
        (color, color_cnt as f32)
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

    pub fn check_color(image: &RgbImage, color: [u8; 3], percentage: f32) -> bool {
        let dim = {
            let (x, y) = image.dimensions();
            (x as f32, y as f32)
        };
        if let Some(majority_color) = Ans::majority_color(image) {
            if majority_color.0 == color &&
                //percentage of color in given image
               (majority_color.1 as f32 / (dim.0 * dim.1)) >= percentage {
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
