use std::{
    marker::Sized,
    path::Path,
};

pub trait Load
    where Self: Sized
{
    fn load<P>(filepath: P) -> std::io::Result<Self>
        where P: AsRef<Path>;
}

pub fn read_entire_file<P>(filepath: P) -> std::io::Result<Vec<u8>>
    where P: AsRef<Path>
{
    use std::io::Read;

    let f = std::fs::File::open(filepath)?;
    let mut v = Vec::new();
    std::io::BufReader::new(f).read_to_end(&mut v)?;

    Ok(v)
}

pub fn write_to_file<P, T>(filepath: P, val: &T) -> std::io::Result<()>
    where P: AsRef<Path>,
          T: Sized
{
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