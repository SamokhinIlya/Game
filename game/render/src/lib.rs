extern crate utils;
extern crate platform;

use utils::clamp;
use platform::graphics::Bitmap;

pub fn fill_rect(dst_bmp: &Bitmap, p0: (i32, i32), p1: (i32, i32), color: Color) {
    for row in dst_bmp.clamped_view(p0, p1) {
        for pxl in row {
            *pxl = color.as_u32();
        }
    }
}

pub fn draw_bmp(dst_bmp: &Bitmap, src_bmp: &Bitmap, p: (i32, i32)) {
    let src0 = (if p.0 < 0 { -p.0 } else { 0 }, if p.1 < 0 { -p.1 } else { 0 });
    let src1 = (src_bmp.width, src_bmp.height);

    let dst0 = p;
    let dst1 = (dst0.0 + src1.0, dst0.1 + src1.1);

    let dst_view = dst_bmp.clamped_view(dst0, dst1);
    let src_view = src_bmp.clamped_view(src0, src1);
    for (dst_row, src_row) in dst_view.zip(src_view) {
        for (dst, src) in dst_row.iter_mut().zip(src_row.iter_mut()) {
            //FIXME: slow!
            let src_color = *src;
            let dst_color = *dst;

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

            *dst = color;
        }
    }
}

#[inline]
pub fn clear(dst_bmp: &Bitmap, color: Color) {
    fill_rect(dst_bmp, (0, 0), dst_bmp.dim(), color);
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
