pub mod text;

use utils::clamp;
use crate::bitmap::Bitmap;
use crate::vector::prelude::*;

pub fn fill_rect(dst_bmp: &Bitmap, mut min: V2i, mut max: V2i, color: Color) {
    use std::mem::swap;

    if min.x > max.x {
        swap(&mut min.x, &mut max.x)
    }
    if min.y > max.y {
        swap(&mut min.y, &mut max.y)
    }

    for row in dst_bmp.clamped_view(min, max) {
        for pxl in row {
            *pxl = color.into();
        }
    }
}

//FIXME: if left or top edge is in bounds,
//       but same edge + thickness is out of bounds,
//       function panics
pub fn draw_rect(
    dst: &mut Bitmap,
    mut min: V2i,
    mut max: V2i,
    color: Color,
    thickness: i32,
) {
    if min.x >= dst.width() || min.y >= dst.height() || max.x < 1 || max.y < 1 {
        return
    }

    let draw_left = min.x >= 0;
    let draw_top = min.y >= 0;
    let draw_right = max.x <= dst.width();
    let draw_bottom = max.y <= dst.height();

    if draw_left && draw_top && draw_right && draw_bottom {
        for t in 0..thickness {
            for x in min.x..max.x {
                dst[(x, min.y + t)] = color.into();
                dst[(x, max.y - t - 1)] = color.into();
            }
            for y in (min.y + 1)..max.y {
                dst[(min.x + t    , y)] = color.into();
                dst[(max.x - t - 1, y)] = color.into();
            }
        }
    } else {
        if !draw_left {
            min.x = 0;
        }
        if !draw_top {
            min.y = 0;
        }
        if !draw_right {
            max.x = dst.width();
        }
        if !draw_bottom {
            max.y = dst.height();
        }

        if draw_left {
            for t in 0..thickness {
                for y in (min.y + 1)..max.y {
                    dst[(min.x + t, y)] = color.into();
                }
            }
        }
        if draw_top {
            for t in 0..thickness {
                for x in min.x..max.x {
                    dst[(x, min.y + t)] = color.into();
                }
            }
        }
        if draw_right {
            for t in 0..thickness {
                for y in (min.y + 1)..max.y {
                    dst[(max.x - t - 1, y)] = color.into();
                }
            }
        }
        if draw_bottom {
            for t in 0..thickness {
                for x in min.x..max.x {
                    dst[(x, max.y - t - 1)] = color.into();
                }
            }
        }
    }
}

// (y - y0) / (y1 - y0) = (x - x0) / (x1 - x0)
// y = (y1 - y0) / (x1 - x0) * (x - x0) + y0
pub fn draw_line(
    dst: &mut Bitmap,
    mut min: V2i,
    mut max: V2i,
    color: Color,
    _thickness: i32,
) {
    //TODO:
    // line clipping
    // horizontal and vertical special cases
    // thickness?

    use std::mem::swap;

    let width = (max.x - min.x) as f32;
    let height = (max.y - min.y) as f32;
    if width.abs() > height.abs() {
        if width.is_sign_negative() {
            swap(&mut min, &mut max);
        }
        let slope = height / width;
        let derr = slope.abs();
        let dy = slope.signum() as i32;

        let mut err = 0.0;
        let mut y = min.y;
        for x in min.x..max.x {
            dst[(x, y)] = color.into();
            err += derr;
            if err >= 0.5 {
                y += dy;
                err -= 1.0;
            }
        }
    } else {
        if height.is_sign_negative() {
            swap(&mut min, &mut max);
        }
        let slope = width / height;
        let derr = slope.abs();
        let dx = slope.signum() as i32;

        let mut err = 0.0;
        let mut x = min.x;
        for y in min.y..max.y {
            dst[(x, y)] = color.into();
            err += derr;
            if err >= 0.5 {
                x += dx;
                err -= 1.0;
            }
        }
    }
}

pub fn draw_bmp(dst: &Bitmap, src: &Bitmap, p: V2i) {
    let src0 = v2!(
        if p.x < 0 { -p.x } else { 0 },
        if p.y < 0 { -p.y } else { 0 },
    );
    let src1 = src.dim();

    let dst0 = p;
    let dst1 = dst0 + src1;

    let dst_view = dst.clamped_view(dst0, dst1);
    let src_view = src.clamped_view(src0, src1);
    for (dst_row, src_row) in dst_view.zip(src_view) {
        for (dst, src) in dst_row.iter_mut().zip(src_row.iter_mut()) {
            let src_color = *src;
            let dst_color = *dst;

            let acoeff: f32 = (src_color >> 24) as f32 / 255.0;

            let sr: i32 = ((src_color & Color::R_MASK) >> 16) as i32;
            let sg: i32 = ((src_color & Color::G_MASK) >> 8) as i32;
            let sb: i32 = (src_color & Color::B_MASK) as i32;

            let dr: i32 = ((dst_color & Color::R_MASK) >> 16) as i32;
            let dg: i32 = ((dst_color & Color::G_MASK) >> 8) as i32;
            let db: i32 = (dst_color & Color::B_MASK) as i32;

            // r = dr + (sr - dr) * acoeff
            let r: u32 = (dr + ((sr - dr) as f32 * acoeff) as i32) as u32;
            let g: u32 = (dg + ((sg - dg) as f32 * acoeff) as i32) as u32;
            let b: u32 = (db + ((sb - db) as f32 * acoeff) as i32) as u32;

            let color: u32 = (r << 16) | (g << 8) | b;

            *dst = color;
        }
    }
}

#[inline]
pub fn clear(dst: &Bitmap, color: Color) {
    fill_rect(dst, v2!(0, 0), dst.dim(), color);
}

#[derive(Copy, Clone)]
pub struct Color {
    data: u32,
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        color.data
    }
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

    pub fn argb(mut a: f32, r: f32, g: f32, b: f32) -> Self {
        let mut color = Self::rgb(r, g, b);
        clamp(&mut a, 0.0, 1.0);
        let alpha = ((a * 255.0).round() as u32) << 24;
        color.data &= !Self::A_MASK;
        color.data |= alpha;
        color
    }

    pub const A_MASK: u32 = 0xFF00_0000;
    pub const R_MASK: u32 = 0x00FF_0000;
    pub const G_MASK: u32 = 0x0000_FF00;
    pub const B_MASK: u32 = 0x0000_00FF;

    pub const TRANSPARENT: Self = Self { data: 0 };
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
        data: Self::A_MASK | 0x7F7F7F,
    };
}
