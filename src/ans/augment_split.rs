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

use ans::label::*;
use ans::return_type::*;
use ans::split_image::*;
use ans::ans_builder::*;
use ans::color_values::ColorValues;
use ans::SplitOffset;
use ans::ImageKind;

pub struct AugmentSplit {
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

impl<'a> AugmentSplit {
    pub fn build(
        img_dir: PathBuf,
        label_type: LabelType,
        split_size: Option<(u32, u32)>,
        split_offset: (Option<SplitOffset>, Option<SplitOffset>),
        img_format: ImgFormat,
        discard_barrier: Option<([u8; 3], f32)>,
        rotation: u8,
        output_real: PathBuf,
        output_mask: Option<PathBuf>
    ) -> AugmentSplit {
        AugmentSplit {
            img_dir: img_dir,
            label_type: label_type,
            split_size: split_size,
            split_offset: split_offset,
            img_format: img_format,
            discard_barrier: discard_barrier,
            rotation: rotation,
            output_real: output_real,
            output_mask: output_mask,
        }
    }

    pub fn get_imgdir(&self) -> PathBuf {
        self.img_dir.clone()
    }

    pub fn get_label_type(&self) -> LabelType {
        self.label_type.clone()
    }
    fn write_to_file(&self, split_image: &SplitImage, line_file: &mut String) {
        let name = self.create_name(&split_image);
        if let Some(ref image) = split_image.image {
            match *image {
                DynamicImage::ImageRgb8(ref image) => {
                    let image_path = self.create_path(&name, ImageKind::Real);
                    let _ = image.save(&image_path);
                    line_file.push_str("/");
                    line_file.push_str(&name);

                    if let Some(ref label) = split_image.label {
                        match *label {
                            Label::Sick => line_file.push_str(" 1"),
                            Label::Healthy => line_file.push_str(" 0"),
                        }
                        line_file.push_str("\n");
                    }
                },
                DynamicImage::ImageLuma8(ref image) => {
                    let image_path = self.create_path(&name, ImageKind::Mask);

                    let dim = self.split_size.unwrap();
                    let mut buffer = ImageBuffer::<Luma<u8>, Vec<u8>>::new(dim.0, dim.1);

                    for (x, y, pixel) in image.enumerate_pixels().filter(|p| p.2.data != [0]) {
                        let mut pixel = pixel.to_luma();
                        pixel.data = [1];
                        buffer.put_pixel(x, y, pixel);
                    };
                    let _ = buffer.save(&image_path);
                },
                _ => {}
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

    fn create_path(&self, name: &str, image_kind: ImageKind) -> PathBuf {
        let path = self.img_dir.clone();
        let mut image_path = path.parent().unwrap().to_path_buf();

        match image_kind {
            ImageKind::Real => {
                image_path.push(self.output_real.clone());
            },
            ImageKind::Mask => {
                if let Some(ref out) = self.output_mask {
                    image_path.push(out.clone());
                } else {
                    panic!("  ...  ")
                }
            }
        }

        DirBuilder::new().recursive(true).create(&image_path).unwrap();
        image_path.push(Path::new(&name[..]));

        image_path
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
    fn random_rotation(&self, split_image: &mut SplitImage) {
        let range = Range::new(1,3);
        let mut rng = thread_rng();
        let rotation = range.ind_sample(&mut rng);

        let image = mem::replace(&mut split_image.image, None);

        if let Some(mut image) = image {
            match rotation {
                1 => image = image.rotate90(),
                2 => image = image.rotate180(),
                3 => image = image.rotate270(),
                _ => {}
            };
            split_image.set_rotation(rotation);
            split_image.set_image(image);
        }
    }

    pub fn build_healthy(&mut self, img_reader: &mut ImgReader) {
        let black_treshhold = 0.35;
        let white_treshhold = 0.8;

        if let Some((x_len, y_len)) = self.split_size {

            let pixels = (x_len * y_len) as f32;

            if let (Some(x_offset), Some(y_offset)) = self.split_offset.clone() {
                let x_offset = SplitOffset::get_value(&x_offset);
                let y_offset = SplitOffset::get_value(&y_offset);
                let mut line_file = String::new();

                for (name, mut img_tuple) in img_reader.img_map.iter_mut() {
                    let real_dim = img_tuple.0.dimensions();

                    for i in (0 .. real_dim.0 - x_len + 1).step_by(x_offset) {

                        for j in (0 .. real_dim.1 - y_len + 1).step_by(y_offset) {
                            let real_crop = img_tuple.0.crop(i, j, x_len, y_len);
                            let mask_crop = img_tuple.1.crop(i, j, x_len, y_len);

                            if let Ok(real_info) = AugmentSplit::get_color(ColorValues::black_rgb(), &real_crop) {

                                if real_info.1 / pixels < black_treshhold {

                                    if let Ok(mask_info) = AugmentSplit::get_color(ColorValues::white_luma(), &mask_crop){
                                        if mask_info.1 / pixels < 1.0 - white_treshhold {
                                            let mut split_image = SplitImage::build(String::from(name.to_str().unwrap()), x_len, y_len, 0, i, j);
                                            split_image.label = Some(Label::Healthy);
                                            split_image.image = Some(real_crop);

                                            self.write_to_file(&split_image, &mut line_file)
                                        }
                                    } else {

                                    }
                                }
                            } else {

                            }
                        }
                    }
                }
                self.write_line_file(line_file)
            }
        }
    }
    pub fn get_color(color: ColorValues, image: &DynamicImage) -> Result<(ColorValues, f32), &str>{
        match *image {
            DynamicImage::ImageLuma8(ref image) => {
                if let ColorValues::LUMA(c) = color {
                    let color_cnt = image.pixels().filter(|x| x.data == c).count();
                    Ok((color, color_cnt as f32))
                } else {
                    Err("Tried to compare [u8; 3] with [u8; 1]")
                }
            },
            DynamicImage::ImageRgb8(ref image) => {
                if let ColorValues::RGB(c) = color {
                    let color_cnt = image.pixels().filter(|x| x.data == c).count();
                    Ok((color, color_cnt as f32))
                } else {
                    Err("Tried to compare [u8; 1] with [u8; 3]")
                }
            },
            _ => {
                Err("unsuppoerted image format")
            },
        }
    }
    pub fn check_color(image: &DynamicImage, color: ColorValues, percentage: f32) -> bool {
        let dim = {
            let (x, y) = image.dimensions();
            (x as f32, y as f32)
        };
        if let Some(majority_color) = AugmentSplit::majority_color(image) {
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
    pub fn majority_color(image: &DynamicImage) -> Option<(ColorValues, usize)> {
        let mut color_map: HashMap<ColorValues, usize> = HashMap::new();
        match *image {
            DynamicImage::ImageLuma8(ref image) => {
                for pixel in image.pixels() {
                    let color_cnt = color_map.entry(ColorValues::luma(pixel.data)).or_insert(0);
                    *color_cnt += 1;
                }
            },
            DynamicImage::ImageRgb8(ref image) => {
                for pixel in image.pixels() {
                    let color_cnt = color_map.entry(ColorValues::rgb(pixel.data)).or_insert(0);
                    *color_cnt += 1;
                }
            },
            _ => {},
        }
        None
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
