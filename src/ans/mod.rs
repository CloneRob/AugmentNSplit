
use std::path::Path;
use img_buffer;

enum LabelType<'a> {
    Img(&'a Path),
    FileName,
    CSV(&'a Path),
}

enum SplitOffset {
    Random,
    Val(u16),
}
pub struct Ans<'a> {
    img_dir: &'a Path,
    label_type: LabelType<'a>,

    split_size: Option<(u16, u16)>,
    //offset for x and y values
    split_offset: (Option<SplitOffset>, Option<SplitOffset>),

    //None for batches meaning single files for each split image
    batches: Option<u16>
}

impl<'a> Ans<'a> {
    pub fn new(config_path: &Path) -> Option<Ans> {
        None
    }
}
