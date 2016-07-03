use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashMap;
use std::collections::hash_map::Entry;



pub mod label;
pub mod return_type;
pub mod split_image;
pub mod color_values;
pub mod augment_split;
pub mod ans_builder;

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
