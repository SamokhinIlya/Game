use utils::clamp;
use crate::bitmap::Bitmap;
use crate::vector::V2i;

pub fn fill_rect(dst_bmp: &Bitmap, p0: (i32, i32), p1: (i32, i32), color: Color) {
    for row in dst_bmp.clamped_view(p0, p1) {
        for pxl in row {
            *pxl = color.into();
        }
    }
}

//TODO: thickness
pub fn draw_rect(dst: &mut Bitmap, mut min: V2i, mut max: V2i, thickness: u8, color: Color) {
    let draw_left = min.x >= 0;
    let draw_top = min.y >= 0;
    let draw_right = max.x < dst.width();
    let draw_bottom = max.y < dst.height();

    if draw_left && draw_top && draw_right && draw_bottom {
        for x in min.x..max.x {
            dst[(x, min.y)] = color.into();
            dst[(x, max.y)] = color.into();
        }
        for y in (min.y + 1)..max.y {
            dst[(min.x    , y)] = color.into();
            dst[(max.x - 1, y)] = color.into();
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
            for y in (min.y + 1)..max.y {
                dst[(min.x    , y)] = color.into();
            }
        }
        if draw_top {
            for x in min.x..max.x {
                dst[(x, min.y)] = color.into();
            }
        }
        if draw_right {
            for y in (min.y + 1)..max.y {
                dst[(max.x - 1, y)] = color.into();
            }
        }
        if draw_bottom {
            for x in min.x..max.x {
                dst[(x, max.y)] = color.into();
            }
        }
    }
}

pub fn draw_bmp(dst_bmp: &Bitmap, src_bmp: &Bitmap, p: (i32, i32)) {
    let src0 = (
        if p.0 < 0 { -p.0 } else { 0 },
        if p.1 < 0 { -p.1 } else { 0 },
    );
    let src1 = src_bmp.dim();

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

use std::collections::HashMap;
use std::path::Path;
use crate::file::Load;

//TODO: if height is smaller that 20 letters are barely visible
pub struct FontBitmaps {
    characters: HashMap<char, Bitmap>,
}

impl Load for FontBitmaps {
    fn load<P>(filepath: P) -> std::io::Result<Self>
        where P: AsRef<Path>
    {
        use rusttype::{point, FontCollection, PositionedGlyph, Scale};

        let font = {
            let file = crate::file::read_entire_file(filepath)?;
            let collection = FontCollection::from_bytes(file).unwrap_or_else(|e| {
                panic!("error constructing a FontCollection from bytes: {}", e);
            });
            collection.into_font().unwrap_or_else(|e| {
                panic!("error turning FontCollection into a Font: {}", e);
            })
        };

        const CHAR_HEIGHT: i32 = 20;

        let scale = Scale::uniform(CHAR_HEIGHT as f32);

        // The origin of a line of text is at the baseline (roughly where
        // non-descending letters sit). We don't want to clip the text, so we shift
        // it down with an offset when laying it out. v_metrics.ascent is the
        // distance between the baseline and the highest edge of any glyph in
        // the font. That's enough to guarantee that there's no clipping.
        let v_metrics = font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent);

        const ALL_SYMBOLS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.,!? ";
        let glyphs: Vec<PositionedGlyph> = font.layout(ALL_SYMBOLS, scale, offset).collect();

        let mut char_bitmaps = HashMap::new();
        for (g, ch) in glyphs.iter().zip(ALL_SYMBOLS.chars()) {
            if let Some(bbox) = g.pixel_bounding_box() {
                let char_width = bbox.max.x - bbox.min.x;
                let mut char_bmp = Bitmap::with_dimensions(char_width, CHAR_HEIGHT)
                    .filled(Color::TRANSPARENT);
                g.draw(|x, y, v| {
                    let x = x as i32;
                    let y = y as i32 + (bbox.min.y as i32);
                    char_bmp[(x, y)] = Color::argb(v, 1.0, 1.0, 1.0).into();
                });
                char_bitmaps.insert(ch, char_bmp);
            }
        }
        char_bitmaps.insert(' ', Bitmap::with_dimensions(CHAR_HEIGHT / 2, CHAR_HEIGHT).filled(Color::TRANSPARENT));

        Ok(FontBitmaps { characters: char_bitmaps })
    }
}

impl FontBitmaps {
    pub fn draw_string(&self, dst: &Bitmap, (mut x, y): (i32, i32), string: &str) {
        for ch in string.chars() {
            let bmp = self.characters.get(&ch)
                .unwrap_or_else(|| panic!("No bitmap for character: {}", ch));
            draw_bmp(dst, bmp, (x, y));
            x += bmp.width();
        }
    }
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
        data: Self::A_MASK | 127 | 127 | 127,
    };
}
