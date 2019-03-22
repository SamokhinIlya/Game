use std::{
    collections::HashMap,
    path::Path,
};
use crate::vector::prelude::*;
use super::{
    Bitmap,
    Color,
    draw_bmp,
};

//TODO: if height is smaller that 20 letters are barely visible
pub struct FontBitmaps {
    chars: HashMap<char, Bitmap>,
    height: i32,
}

impl FontBitmaps {
    #[inline(always)]
    pub fn height(&self) -> i32 { self.height }

    /// Draws string of text to the dst `Bitmap`
    /// 
    /// Returns width of drawn string in pixels
    pub fn draw_string(&self, dst: &Bitmap, V2i { x, y }: V2i, s: &str) -> i32 {
        let mut current_x = x;
        for bmp in self.to_bitmaps(s) {
            draw_bmp(dst, bmp, (current_x, y));
            current_x += bmp.width();
        }
        current_x - x
    }

    pub fn width(&self, s: &str) -> i32 {
        self.to_bitmaps(s).map(|bmp| bmp.width()).sum()
    }

    fn to_bitmaps<'res, 'a, 'b>(&'a self, s: &'b str) -> impl Iterator<Item = &'res Bitmap>
        where 'a: 'res, 'b: 'res
    {
        s.chars().map(move |c| self.chars.get(&c).unwrap_or_else(
            || panic!("No bitmap for character: {}", c)))
    }

    pub fn new<P>(filepath: P, height: i32) -> std::io::Result<Self>
        where P: AsRef<Path>
    {
        use rusttype::{point, FontCollection, PositionedGlyph, Scale};

        let font = {
            let file = crate::file::read_entire_file(filepath)?;
            let collection = FontCollection::from_bytes(file)
                .unwrap_or_else(|e| panic!("error constructing a FontCollection from bytes: {}", e));
            collection.into_font()
                .unwrap_or_else(|e| panic!("error turning FontCollection into a Font: {}", e))
        };

        let scale = Scale::uniform(height as f32);

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
                let width = bbox.max.x - bbox.min.x;
                let mut bmp = Bitmap::with_dimensions(width, height).filled(Color::TRANSPARENT);
                g.draw(|x, y, v| {
                    let x = x as i32;
                    let y = y as i32 + (bbox.min.y as i32);
                    bmp[(x, y)] = Color::argb(v, 1.0, 1.0, 1.0).into();
                });

                char_bitmaps.insert(ch, bmp);
            }
        }
        let space = Bitmap::with_dimensions(height / 2, height).filled(Color::TRANSPARENT);
        char_bitmaps.insert(' ', space);

        Ok(FontBitmaps {
            chars: char_bitmaps,
            height,
        })
    }
}