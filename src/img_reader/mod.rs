use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::ffi::OsString;
use std::collections::HashMap;
use image;

pub struct ImgReader {
    img_map: HashMap<OsString, (image::DynamicImage, image::DynamicImage)>,
    num_of_images: usize,
}

/*
impl ImageReader {
    pub fn new(path: &'a Path) -> ImageReader {
        let path_vec = list_images(path);

        unimplemented!()
    }
}
*/

fn list_images<'a>(path: &'a Path) -> Vec<(OsString, PathBuf)> {
    let paths = fs::read_dir(path).unwrap();
    let mut path_vec = Vec::new();
    paths.map(|x| path_vec.push((x.unwrap().file_name(), x.unwrap().path())));
    path_vec
}

fn create_img_map(path_vec: Vec<(OsString, PathBuf)>) -> HashMap<OsString, (image::DynamicImage, image::DynamicImage)> {
    let mut img_map = HashMap::new();

    unimplemented!()
}
