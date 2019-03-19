use std::{
    fs::File,
    io::BufReader,
    marker::Sized,
};

pub use std::path::Path;
pub use std::io;

pub trait Load
    where Self: Sized
{
    fn load<P>(filepath: P) -> io::Result<Self>
        where P: AsRef<Path>;
}

pub trait Save
    where Self: Sized
{
    fn save<P>(&self, filepath: P) -> io::Result<()>
        where P: AsRef<Path>;
}

pub fn read_entire_file<P>(filepath: P) -> io::Result<Vec<u8>>
    where P: AsRef<Path>
{
    let f = File::open(filepath)?;
    let mut v = Vec::new();

    use std::io::Read;
    BufReader::new(f).read_to_end(&mut v)?;

    Ok(v)
}

pub fn write_bytes_to_file<P>(filepath: P, bytes: &[u8]) -> io::Result<()>
    where P: AsRef<Path>
{
    //TODO: BufWrite?
    use std::io::Write;
    File::create(filepath)?.write_all(bytes)
}