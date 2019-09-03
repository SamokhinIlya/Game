use std::{
    mem,
    ptr,
    ops::{Index, IndexMut},
    iter::Iterator,
    slice,
};
use crate::{
    file::{prelude::*, read_entire_file},
    geom::vector::prelude::*,
    render::Color,
};

#[derive(Debug)]
pub struct Bitmap {
    data: *mut u32,
    width: i32,
    height: i32,
}

impl Drop for Bitmap {
    fn drop(&mut self) {
        use std::alloc::{dealloc, Layout};
        unsafe {
            dealloc(
                self.data as *mut u8,
                Layout::from_size_align_unchecked(
                    mem::size_of::<u32>() * self.width as usize * self.height as usize,
                    mem::align_of::<u32>(),
                )
            );
        }
    }
}

impl Index<(i32, i32)> for Bitmap {
    type Output = u32;
    fn index(&self, (x, y): (i32, i32)) -> &Self::Output {
        unsafe {
            &*self.ptr_mut_at(x as usize, y as usize)
        }
    }
}

impl IndexMut<(i32, i32)> for Bitmap {
    fn index_mut(&mut self, (x, y): (i32, i32)) -> &mut u32 {
        unsafe {
            &mut *self.ptr_mut_at(x as usize, y as usize)
        }
    }
}


impl Bitmap {
    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }
    pub fn dim(&self) -> V2i { (self.width, self.height).into() }

    pub fn with_dimensions(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0);

        #[allow(clippy::cast_ptr_alignment)]
        let data = unsafe {
            use std::alloc::{alloc, Layout};
            alloc(Layout::from_size_align_unchecked(
                mem::size_of::<u32>() * width as usize * height as usize,
                mem::align_of::<u32>(),
            )) as *mut u32
        };

        Self { data, width, height }
    }

    pub fn filled(mut self, color: Color) -> Self {
        for p in self.as_mut_slice() {
            *p = color.into()
        }
        self
    }

    pub fn as_slice(&self) -> &[u32] {
        unsafe {
            slice::from_raw_parts(self.data, self.width as usize * self.height as usize)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u32] {
        unsafe {
            slice::from_raw_parts_mut(self.data, self.width as usize * self.height as usize)
        }
    }

    pub fn clamped_view(&self, mut top_left: V2i, mut bottom_right: V2i) -> BitmapView {
        //TODO: rectangle type and contains method
        top_left.x = utils::clamp(top_left.x, 0, self.width);
        top_left.y = utils::clamp(top_left.y, 0, self.height);
        bottom_right.x = utils::clamp(bottom_right.x, 0, self.width);
        bottom_right.y = utils::clamp(bottom_right.y, 0, self.height);

        let ptr = unsafe {
            self.data.add((top_left.y * self.width + top_left.x) as usize)
        };
        let height = (bottom_right.y - top_left.y) as isize;

        BitmapView {
            ptr,
            end: unsafe { ptr.offset(height * self.width as isize) },
            width: (bottom_right.x - top_left.x) as usize,
            bmp: self,
        }
    }

    unsafe fn ptr_mut_at(&self, x: usize, y: usize) -> *mut u32 {
        assert!(
            x < self.width as usize && y < self.height as usize,
            "Bitmap index out of bounds. (width, height) = {:?}, (x, y) = {:?}",
            self.dim(), (x, y), 
        );
        self.data.add(y * self.width as usize + x)
    }

    #[allow(clippy::items_after_statements)]
    pub fn load(filepath: impl AsRef<Path>) -> Result {
        let file_extension = filepath.as_ref().extension()
            .ok_or(BitmapLoadError::NoFileExtension)?;

        return match file_extension {
            ext if ext == "bmp" => load_bmp(filepath),
            ext if ext == "png" => load_png(filepath),
            _ => Err(BitmapLoadError::UnsupportedFormat),
        };

        fn load_png(filepath: impl AsRef<Path>) -> Result {
            let mut png = lodepng::decode32(read_entire_file(filepath)?)?;

            for x in &mut png.buffer {
                let rgba = *x;
                *x = rgb::RGBA8 {
                    r: rgba.b,
                    g: rgba.g,
                    b: rgba.r,
                    a: rgba.a,
                };
            }

            // FIXME: what will drop do
            #[allow(clippy::cast_ptr_alignment)]
            let data = png.buffer.as_mut_ptr() as *mut u32;
            let width = png.width as i32;
            let height = png.height as i32;
            mem::forget(png);

            Ok(Bitmap { data, width, height })
        }

        fn load_bmp(filepath: impl AsRef<Path>) -> Result {
            use file_header::bmp::*;

            let file = read_entire_file(filepath)?;
            let header: BitmapHeader = unsafe { ptr::read_unaligned(file.as_ptr() as *const _) };
            assert!(header.BITMAPFILEHEADER.bfType == unsafe { mem::transmute(*b"BM") });

            let bmp_data = {
                //FIXME: alloc
                let mut vec = Vec::<u32>::with_capacity(
                    header.BITMAPV5HEADER.bV5SizeImage as usize / mem::size_of::<u32>()
                );
                let ptr = vec.as_mut_ptr();
                mem::forget(vec);
                ptr
            };
            let bmp_width = header.BITMAPV5HEADER.bV5Width;
            let bmp_height = header.BITMAPV5HEADER.bV5Height;

            let mut dst_row: *mut u32 = bmp_data;

            #[allow(clippy::cast_ptr_alignment)]
            let mut src_row: *mut u32 = unsafe {
                let end_row_offset = header.BITMAPFILEHEADER.bfOffBits as usize
                    + ((bmp_height - 1) * bmp_width) as usize * mem::size_of::<u32>();
                file.as_ptr().add(end_row_offset) as *mut u32
            };

            unsafe {
                for _y in 0..bmp_height {
                    let mut dst = dst_row;
                    let mut src = src_row;
                    for _x in 0..bmp_width {
                        ptr::write_unaligned(dst, *src);
                        dst = dst.add(1);
                        src = src.add(1);
                    }
                    dst_row = dst_row.add(bmp_width as usize);
                    src_row = src_row.sub(bmp_width as usize);
                }
            }

            Ok(Bitmap {
                data: bmp_data,
                width: bmp_width,
                height: bmp_height,
            })
        }
    }
}

pub type Result = std::result::Result<Bitmap, BitmapLoadError>;

#[derive(Debug)]
pub enum BitmapLoadError {
    IoError(std::io::Error),
    PngError(&'static str),
    UnsupportedFormat,
    NoFileExtension,
}

impl From<std::io::Error> for BitmapLoadError {
    fn from(err: std::io::Error) -> Self {
        BitmapLoadError::IoError(err)
    }
}

impl From<lodepng::ffi::Error> for BitmapLoadError {
    fn from(err: lodepng::ffi::Error) -> Self {
        BitmapLoadError::PngError(err.as_str())
    }
}

pub struct BitmapView<'a> {
    ptr: *mut u32,
    end: *mut u32,
    width: usize,
    bmp: &'a Bitmap,
}

impl<'a> Iterator for BitmapView<'a> {
    type Item = &'a mut [u32];
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr < self.end {
            unsafe {
                let slice = slice::from_raw_parts_mut(self.ptr, self.width);
                self.ptr = self.ptr.add(self.bmp.width as usize);
                Some(slice)
            }
        } else {
            None
        }
    }
}

impl From<platform::graphics::WindowBuffer> for Bitmap {
    fn from(window_buffer: platform::graphics::WindowBuffer) -> Self {
        #[allow(clippy::cast_ptr_alignment)]
        Self {
            data: window_buffer.data as *mut u32,
            width: window_buffer.width,
            height: window_buffer.height,
        }
    }
}

mod file_header {
    #[allow(non_snake_case)]
    pub mod bmp {
        #[repr(C, packed)]
        #[derive(Copy, Clone, Debug)]
        pub struct BitmapHeader {
            pub BITMAPFILEHEADER: BITMAPFILEHEADER,
            pub BITMAPV5HEADER: BITMAPV5HEADER,
        }

        #[repr(C, packed)]
        #[derive(Copy, Clone, Debug)]
        pub struct BITMAPFILEHEADER {
            pub bfType: u16,
            pub bfSize: u32,
            pub bfReserved1: u16,
            pub bfReserved2: u16,
            pub bfOffBits: u32,
        }

        #[repr(C, packed)]
        #[derive(Copy, Clone, Debug)]
        pub struct BITMAPV5HEADER {
            pub bV5Size: u32,
            pub bV5Width: i32,
            pub bV5Height: i32,
            pub bV5Planes: u16,
            pub bV5BitCount: u16,
            pub bV5Compression: u32,
            pub bV5SizeImage: u32,
            pub bV5XPelsPerMeter: i32,
            pub bV5YPelsPerMeter: i32,
            pub bV5ClrUsed: u32,
            pub bV5ClrImportant: u32,
            pub bV5RedMask: u32,
            pub bV5GreenMask: u32,
            pub bV5BlueMask: u32,
            pub bV5AlphaMask: u32,
            /*
            pub bV5CSType: u32       ,
            pub bV5Endpoints: CIEXYZTRIPLE,
            pub bV5GammaRed: u32       ,
            pub bV5GammaGreen: u32       ,
            pub bV5GammaBlue: u32       ,
            pub bV5Intent: u32       ,
            pub bV5ProfileData: u32       ,
            pub bV5ProfileSize: u32       ,
            pub bV5Reserved: u32       ,
            */
        }
    }
}
