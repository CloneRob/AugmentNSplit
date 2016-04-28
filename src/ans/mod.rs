

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::reader::{EventReader, XmlEvent};

use img_reader::LabelType;

enum SplitOffset {
    Random,
    Val(u16),
}
pub struct Ans<'a> {
    img_dir: &'a Path,
    label_type: LabelType<'a>,

    split_size: Option<(u16, u16)>,
    // offset for x and y values
    split_offset: (Option<SplitOffset>, Option<SplitOffset>),

    // None for batches meaning single files for each split image
    batches: Option<u16>,
}

impl<'a> Ans<'a> {
    pub fn from_config(config_path: &Path) -> Option<Ans> {
        let file = File::open(config_path).unwrap();
        let file = BufReader::new(file);

        let parser = EventReader::new(file);
        for e in parser {
            match e {
                Ok(XmlEvent::StartElement { name,  .. }) => {
                    println!("{}" , name);
                }
                Ok(XmlEvent::EndElement { name }) => {
                    println!("{}", name);
                }
                Ok(XmlEvent::CData(s)) => println!("{:?}", s),
                Ok(XmlEvent::Characters(s)) => println!("{:?}", s),
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        unimplemented!()
    }
}
