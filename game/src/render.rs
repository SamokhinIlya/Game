pub mod text;
pub mod color;
pub mod bitmap;

use std::mem::swap;
use bitmap::Bitmap;
use crate::vector::prelude::*;

pub use color::Color;

pub fn fill_rect(dst_bmp: &Bitmap, mut min: V2i, mut max: V2i, color: Color) {
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

pub fn clear(dst: &Bitmap, color: Color) {
    fill_rect(dst, v2!(0, 0), dst.dim(), color);
}
