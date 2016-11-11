use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use image;

#[derive(Clone)]
pub enum LabelType {
    Img(PathBuf),
    FileName,
    CSV(PathBuf),
}

pub struct ImgReaderGeneric<D, L> {
    num_of_images: usize,
    pub img_map: HashMap<String, (D, L)>,
}


pub struct ImgReader {
    num_of_images: usize,
    pub img_map: HashMap<String, (image::DynamicImage, image::DynamicImage)>,
}

impl ImgReader {
    pub fn new<'a, 'b: 'a>(img_path: PathBuf, label_type: LabelType) -> ImgReader {
        let training_map = image_map(img_path);
        let mut label_map: HashMap<String, image::DynamicImage> = match label_type {
            // TODO Currently this only works for labels in the form of an image, which is my current
            // use case. Support for the other fields in the LabelType will be added later
            LabelType::Img(p) => image_map(p),
            _ => unimplemented!(),
        };

        let img_map = {
            let mut img_map = HashMap::new();
            for (name, training_img) in training_map {
                match label_map.remove(&name) {
                    Some(label_img) => {
                        // println!("Inserting {:?} into image map", name);
                        img_map.insert(name, (training_img, label_img.clone()));
                    }
                    None => {
                        panic!("Could not find a corresponding Label for Image {}", name);
                    }
                }
            }
            img_map
        };

        ImgReader {
            num_of_images: img_map.len(),
            img_map: img_map,
        }
    }

    pub fn get_num_of_images(&self) -> usize {
        self.num_of_images
    }
}

fn image_map<'a>(img_path: PathBuf) -> HashMap<String, image::DynamicImage> {
    let dir_entries = fs::read_dir(img_path)
        .expect("The specified path given to fn image_map() doesn't seem to exist");
    let mut path_map = HashMap::new();

    for d in dir_entries {
        let dir_entry = d.unwrap();

        let img_name = dir_entry.file_name().into_string().unwrap();
        let img_file = image::open(dir_entry.path());

        if let Ok(image) = img_file {
            path_map.insert(img_name, image);
        } else {
            panic!("Error in fn image_map(); Could not read Image {:?}",
                   img_name);
        }
    }
    path_map
}
