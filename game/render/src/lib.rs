extern crate utils;
extern crate platform;

use utils::clamp;
use platform::graphics::Bitmap;

pub fn fill_rect(
    dst_bmp: &Bitmap,
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    color: Color,
) {
    clamp(&mut x0, 0, dst_bmp.width);
    clamp(&mut y0, 0, dst_bmp.height);
    clamp(&mut x1, 0, dst_bmp.width);
    clamp(&mut y1, 0, dst_bmp.height);

    let u32_color = color.as_u32();

    let mut row: *mut u32 = unsafe { dst_bmp.data.add((y0 * dst_bmp.width) as usize) };
    for _y in y0..y1 {
        let mut pixel = unsafe { row.add(x0 as usize) };
        for _x in x0..x1 {
            unsafe {
                *pixel = u32_color;
                pixel = pixel.add(1);
            }
        }
        row = unsafe { row.add(dst_bmp.width as usize) };
    }
}

//FIXME: artifacts at the top-left edges of the screen,
//       when drawn bmp intersects them
pub fn draw_bmp(
    dst_bmp: &Bitmap,
    src_bmp: &Bitmap,
    mut dst_x0: i32,
    mut dst_y0: i32,
) {
    let src_width = src_bmp.width as usize;
    let dst_width = dst_bmp.width as usize;

    let src_x0 = if dst_x0 < 0 { -dst_x0 } else { 0 };
    let src_y0 = if dst_y0 < 0 { -dst_y0 } else { 0 };
    let src_x1 = src_bmp.width;
    let src_y1 = src_bmp.height;

    let mut dst_x1 = dst_x0 + src_x1;
    let mut dst_y1 = dst_y0 + src_y1;

    clamp(&mut dst_x0, 0, dst_bmp.width);
    clamp(&mut dst_y0, 0, dst_bmp.height);
    clamp(&mut dst_x1, 0, dst_bmp.width);
    clamp(&mut dst_y1, 0, dst_bmp.height);

    let mut src_row = unsafe { src_bmp.data.add(src_y0 as usize * src_width) };
    let mut dst_row = unsafe { dst_bmp.data.add(dst_y0 as usize * dst_width) };
    for _y in dst_y0..dst_y1 {
        let mut src = unsafe { src_row.add(src_x0 as usize) };
        let mut dst = unsafe { dst_row.add(dst_x0 as usize) };
        for _x in dst_x0..dst_x1 {
            //FIXME: slow!
            let src_color = unsafe { *src };
            let dst_color = unsafe { *dst };

            let acoeff: f32 = (src_color >> 24) as f32 / 255.0;

            let sr: i32 = ((src_color & Color::R_MASK) >> 16) as i32;
            let sg: i32 = ((src_color & Color::G_MASK) >> 8) as i32;
            let sb: i32 = (src_color & Color::B_MASK) as i32;

            let dr: i32 = ((dst_color & Color::R_MASK) >> 16) as i32;
            let dg: i32 = ((dst_color & Color::G_MASK) >> 8) as i32;
            let db: i32 = (dst_color & Color::B_MASK) as i32;

            //XXX: r = dr + (sr - dr) * acoeff
            let r: u32 = (dr + ((sr - dr) as f32 * acoeff) as i32) as u32;
            let g: u32 = (dg + ((sg - dg) as f32 * acoeff) as i32) as u32;
            let b: u32 = (db + ((sb - db) as f32 * acoeff) as i32) as u32;

            let color: u32 = (r << 16) | (g << 8) | b;

            unsafe {
                *dst = color;
                src = src.add(1);
                dst = dst.add(1);
            }
        }
        unsafe {
            src_row = src_row.add(src_width);
            dst_row = dst_row.add(dst_width);
        }
    }
}

#[inline]
pub fn clear(dst_bmp: &Bitmap, color: Color) {
    fill_rect(dst_bmp, 0, 0, dst_bmp.width, dst_bmp.height, color);
}

#[derive(Copy, Clone)]
pub struct Color {
    data: u32,
}

#[allow(dead_code)]
impl Color {
    pub fn rgb(mut r: f32, mut g: f32, mut b: f32) -> Self {
        clamp(&mut r, 0.0, 1.0);
        clamp(&mut g, 0.0, 1.0);
        clamp(&mut b, 0.0, 1.0);
        let data = 0xFF00_0000
            | ((r * 255.0).round() as u32) << 16
            | ((g * 255.0).round() as u32) << 8
            | (b * 255.0).round() as u32;

        Self { data }
    }

    pub fn as_u32(self) -> u32 {
        self.data
    }

    pub const A_MASK: u32 = 0xFF00_0000;
    pub const R_MASK: u32 = 0x00FF_0000;
    pub const G_MASK: u32 = 0x0000_FF00;
    pub const B_MASK: u32 = 0x0000_00FF;

    pub const BLACK: Self = Self { data: Self::A_MASK };
    pub const WHITE: Self = Self {
        data: Self::A_MASK | Self::R_MASK | Self::G_MASK | Self::B_MASK,
    };
    pub const YELLOW: Self = Self {
        data: Self::A_MASK | Self::R_MASK | Self::G_MASK,
    };
    pub const RED: Self = Self {
        data: Self::A_MASK | Self::R_MASK,
    };
    pub const PURPLE: Self = Self {
        data: Self::A_MASK | Self::R_MASK | Self::B_MASK,
    };
    pub const GREY: Self = Self {
        data: Self::A_MASK | 127 | 127 | 127,
    };
}
