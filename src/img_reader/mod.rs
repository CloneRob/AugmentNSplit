use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::ffi::OsString;
use std::collections::HashMap;
use image;

pub enum LabelType {
    Img(PathBuf),
    FileName,
    CSV(PathBuf),
}

pub struct ImgReader {
    num_of_images: usize,
    img_map: HashMap<OsString, (image::DynamicImage, image::DynamicImage)>,
}

impl ImgReader {
    pub fn new<'a, 'b: 'a>(img_path: PathBuf, label_type: LabelType) -> ImgReader {
        let training_map = image_map(img_path);
        let label_map: HashMap<OsString, image::DynamicImage> = match label_type {
            // TODO Currently this only works for labels in the form of an image, which is my current
            // use case. Support for the other fields in the LabelType will be added later
            LabelType::Img(p) => image_map(p),
            _ => unimplemented!(),
        };

        let img_map = {
            let mut img_map = HashMap::new();
            for (name, training_img) in training_map {
                match label_map.get(&name) {
                    Some(label_img) => {
                        img_map.insert(name, (training_img, label_img.clone()));
                    }
                    None => {
                        if let Ok(name_as_string) = name.into_string() {
                            panic!("Could not find a corresponding Label for Image {}",
                                   name_as_string);
                        }
                    }
                }
            }
            img_map
        };

        ImgReader {
            num_of_images: img_map.capacity(),
            img_map: img_map,
        }
    }
}

fn image_map<'a>(img_path: PathBuf) -> HashMap<OsString, image::DynamicImage> {
    let dir_entries = fs::read_dir(img_path)
                          .expect("The specified path given to fn image_map() doesn't seem to \
                                   exist");
    let mut path_map = HashMap::new();

    for d in dir_entries {
        let dir_entry = d.unwrap();

        let img_name = dir_entry.file_name();
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
