

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use xml::reader::{EventReader, XmlEvent, Error};

use img_reader::LabelType;

enum SplitOffset {
    Random,
    Val(u16),
}

struct AnsPathBuilder {
    img_dir: Option<PathBuf>,
    label_type: Option<LabelType>,
}

impl<'a> AnsPathBuilder {
    fn new() -> AnsPathBuilder {
        AnsPathBuilder {
            img_dir: None,
            label_type: None,
        }
    }
    fn set_img_dir(&mut self, path: PathBuf){
        self.img_dir = Some(path);
    }

    fn set_label_type(&mut self, label_type: LabelType) {
        self.label_type = Some(label_type);
    }
}

pub struct Ans<'a> {
    img_dir: &'a Path,
    label_type: LabelType,

    split_size: Option<(u16, u16)>,
    // offset for x and y values
    split_offset: (Option<SplitOffset>, Option<SplitOffset>),

    // None for batches meaning single files for each split image
    batches: Option<u16>,
}

impl<'a> Ans<'a> {
    fn set_split_size(&mut self, x: u16, y: u16) {
        self.split_size = Some((x, y));
    }
    fn set_split_offset(&mut self, x: Option<u16>, y: Option<u16>) {
        match (x, y) {
            (Some(x_val), Some(y_val)) => self.split_offset = (Some(SplitOffset::Val(x_val)), Some(SplitOffset::Val(y_val))),
            (Some(x_val), None) => self.split_offset = (Some(SplitOffset::Val(x_val)), Some(SplitOffset::Random)),
            (None, Some(y_val)) => self.split_offset = (Some(SplitOffset::Random), Some(SplitOffset::Val(y_val))),
            (None, None) => self.split_offset = (Some(SplitOffset::Random), Some(SplitOffset::Random)),
        }
    }

    fn parse_img_dir(xml_events: &mut EventReader<BufReader<File>>, path_builder: &mut AnsPathBuilder) {
        while let Ok(xml_event) = xml_events.next() {
            match xml_event {
                XmlEvent::CData(s) => {
                    path_builder.set_img_dir(PathBuf::from(s));
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == "img_dir" {
                        break
                    }
                }
                _ => panic!("Unknown branch found in <img_dir>, please check your configuration")
            }
        }
    }

    fn parse_label_type(xml_events: &mut EventReader<BufReader<File>>, path_builder: &mut AnsPathBuilder) {
        while let Ok(xml_event) = xml_events.next() {
            match xml_event {
                XmlEvent::StartElement { name, .. } => {
                    match &name.local_name[..] {
                        "label_type" => {

                        },
                        "label_dir"  => {

                        },
                        _ => unimplemented!(),
                    }
                }
                XmlEvent::EndElement { name, .. } => {
                    if name.local_name == "label" {
                        break
                    }
                }
                _ => unimplemented!()
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
                XmlEvent::StartElement {name,  .. } => {
                    match &name.local_name[..] {
                        "img_dir" => Ans::parse_img_dir(&mut parser, &mut ans_path_builder),
                        /*
                        "label" =>,
                        "split" =>,
                        "augment" =>,
                        */
                        _ => unimplemented!()
                    }
                }
                _ => print!("test")
            }
        }
        unimplemented!()
    }
}
