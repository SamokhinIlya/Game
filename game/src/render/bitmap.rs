use std::{
    mem::{self, size_of},
    ptr,
    ops::{Index, IndexMut},
    iter::Iterator,
    slice,
};
use crate::file::{Load, read_entire_file};
use crate::vector::prelude::*;
use crate::render::Color;

#[derive(Debug)]
pub struct Bitmap {
    data: *mut u32,
    width: i32,
    height: i32,
}

impl Drop for Bitmap {
    fn drop(&mut self) {
        let capacity = (self.width * self.height) as usize;
        unsafe {
            let _ = Vec::from_raw_parts(self.data, capacity, capacity);
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
    pub fn dim(&self) -> V2i { v2!(self.width, self.height) }

    pub fn with_dimensions(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0);

        let data = {
            let mut vec = Vec::<u32>::with_capacity(width as usize * height as usize);
            let ptr = vec.as_mut_ptr();
            mem::forget(vec);
            ptr
        };

        Self { data, width, height }
    }

    pub fn filled(self, color: Color) -> Self {
        let slice = unsafe {
            slice::from_raw_parts_mut(self.data, (self.width * self.height) as usize)
        };
        for p in slice {
            *p = color.into()
        }
        self
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

use std::path::Path;

impl Load for Bitmap {
    #[allow(non_snake_case, clippy::cast_ptr_alignment)]
    fn load<P: AsRef<Path>>(filepath: P) -> std::io::Result<Self> {
        let file = read_entire_file(filepath)?;

        #[repr(C, packed)]
        #[derive(Copy, Clone, Debug)]
        struct BITMAPFILEHEADER {
            bfType: u16,
            bfSize: u32,
            bfReserved1: u16,
            bfReserved2: u16,
            bfOffBits: u32,
        }

        #[repr(C, packed)]
        #[derive(Copy, Clone, Debug)]
        struct BITMAPV5HEADER {
            bV5Size: u32,
            bV5Width: i32,
            bV5Height: i32,
            bV5Planes: u16,
            bV5BitCount: u16,
            bV5Compression: u32,
            bV5SizeImage: u32,
            bV5XPelsPerMeter: i32,
            bV5YPelsPerMeter: i32,
            bV5ClrUsed: u32,
            bV5ClrImportant: u32,
            bV5RedMask: u32,
            bV5GreenMask: u32,
            bV5BlueMask: u32,
            bV5AlphaMask: u32,
            /*
            bV5CSType: u32       ,
            bV5Endpoints: CIEXYZTRIPLE,
            bV5GammaRed: u32       ,
            bV5GammaGreen: u32       ,
            bV5GammaBlue: u32       ,
            bV5Intent: u32       ,
            bV5ProfileData: u32       ,
            bV5ProfileSize: u32       ,
            bV5Reserved: u32       ,
            */
        }

        #[repr(C, packed)]
        #[derive(Copy, Clone, Debug)]
        struct BitmapHeader {
            BITMAPFILEHEADER: BITMAPFILEHEADER,
            BITMAPV5HEADER: BITMAPV5HEADER,
        };

        let header = unsafe {
            ptr::read_unaligned(file.as_ptr() as *const BitmapHeader)
        };
        const BM: u16 = ('B' as u16) | ('M' as u16) << 8;
        assert!(header.BITMAPFILEHEADER.bfType == BM);

        let bmp_data = {
            let mut vec = Vec::<u32>::with_capacity(header.BITMAPV5HEADER.bV5SizeImage as usize / size_of::<u32>());
            let ptr = vec.as_mut_ptr();
            mem::forget(vec);
            ptr
        };
        let bmp_width = header.BITMAPV5HEADER.bV5Width;
        let bmp_height = header.BITMAPV5HEADER.bV5Height;

        let mut dst_row: *mut u32 = bmp_data;
        let mut src_row: *mut u32 = unsafe {
            file.as_ptr().add(
                header.BITMAPFILEHEADER.bfOffBits as usize
                    + ((bmp_height - 1) * bmp_width) as usize * size_of::<u32>()
            ) as *mut u32
        };
        unsafe {
            for _y in 0..bmp_height {
                let mut dst = dst_row;
                let mut src = src_row; 
                for _x in 0..bmp_width {
                    *dst = *src;
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

impl From<platform::graphics::WindowBuffer> for Bitmap {
    #[allow(clippy::cast_ptr_alignment)]
    fn from(window_buffer: platform::graphics::WindowBuffer) -> Self {
        Self {
            data: window_buffer.data as *mut u32,
            width: window_buffer.width,
            height: window_buffer.height,
        }
    }
}