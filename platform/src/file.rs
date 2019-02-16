// TODO: this is platform-independent and should not be there
use core::{
    marker::Sized,
};
use std::path::Path;

pub trait Load
    where Self: Sized
{
    fn load<P: AsRef<Path>>(filepath: P) -> std::io::Result<Self>;
}

pub fn read_entire_file<P: AsRef<Path>>(filepath: P) -> std::io::Result<Vec<u8>> {
    use std::io::Read;

    let f = std::fs::File::open(filepath)?;
    let mut v = Vec::new();
    std::io::BufReader::new(f).read_to_end(&mut v)?;
    Ok(v)
}

pub fn write_to_file<T: Sized, P: AsRef<Path>>(filepath: P, val: &T) -> std::io::Result<()> {
    use std::io::Write;

    let mut f = std::fs::File::create(filepath)?;
    let s = unsafe {
        std::slice::from_raw_parts(
            val as *const _ as *const u8,
            std::mem::size_of::<T>(),
        )
    };
    f.write_all(s)
}