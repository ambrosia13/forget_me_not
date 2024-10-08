use std::path::Path;

pub mod buffer;
pub mod preprocess;

pub fn name_from_path<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}
