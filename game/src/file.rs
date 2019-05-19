use std::{
    fs::File,
    io::{self, BufReader},
    marker::Sized,
    path::Path,
};

pub mod prelude {
    pub use super::{Save, Load};
    pub use std::{path::Path, io};
}

pub trait Load: Sized {
    fn load(filepath: impl AsRef<Path>) -> io::Result<Self>;
}

pub trait Save: Sized {
    fn save(&self, filepath: impl AsRef<Path>) -> io::Result<()>;
}

pub fn read_entire_file(filepath: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    use std::io::Read;
    let mut v = Vec::new();
    BufReader::new(File::open(filepath)?).read_to_end(&mut v)?;
    Ok(v)
}

pub fn write_bytes_to_file(filepath: impl AsRef<Path>, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    File::create(filepath)?.write_all(bytes)
}