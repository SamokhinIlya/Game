use core::{
    mem::size_of,
    ptr,
    convert::From,
};
use crate::{
    memory,
    file,
    file::{
        File,
        Load,
    },
};

pub struct Bitmap {
    pub data: *mut u32,
    pub width: i32,
    pub height: i32,
}

impl Bitmap {
    pub fn with_dimensions(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0);

        #[allow(clippy::cast_ptr_alignment)]
        let data = unsafe {
            memory::allocate_bytes(width as usize * height as usize * size_of::<u32>())
                as *mut u32
        };
        Self { data, width, height }
    }

    #[inline]
    pub fn dim(&self) -> (i32, i32) { (self.width, self.height) }
}

impl Load for Bitmap {
    fn load(filepath: &str) -> Result<Self, file::LoadErr> {
        match file::File::read(filepath) {
            Ok(file) => Ok(Bitmap::from(file)),
            Err(err) => Err(file::LoadErr::FileErr(err)),
        }
    }
}

//TODO: remove all hardcoding
impl From<File> for Bitmap {
    #[allow(non_snake_case, clippy::cast_ptr_alignment)]
    fn from(file: File) -> Self {
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

        let header = unsafe { ptr::read(file.data as *mut BitmapHeader) };
        let BM: u16 = ('B' as u16) | ('M' as u16) << 8;
        assert!(header.BITMAPFILEHEADER.bfType == BM);

        let bmp_size = header.BITMAPV5HEADER.bV5SizeImage as usize;
        let bmp_data = unsafe { memory::allocate_bytes(bmp_size) as *mut u32 };
        let bmp_width = header.BITMAPV5HEADER.bV5Width;
        let bmp_height = header.BITMAPV5HEADER.bV5Height;

        let mut dst_row: *mut u32 = bmp_data;
        let mut src_row: *mut u32 = unsafe {
            file.data
                .add(header.BITMAPFILEHEADER.bfOffBits as usize
                      + ((bmp_height - 1) * bmp_width) as usize * size_of::<u32>())
                as *mut u32
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

        Bitmap {
            data: bmp_data,
            width: bmp_width,
            height: bmp_height,
        }
    }
}