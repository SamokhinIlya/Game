use std::{
    collections::HashMap,
    path::Path,
};
use crate::geom::vector::prelude::*;
use super::{
    Bitmap,
    Color,
    draw_bmp,
};

pub struct FontBitmaps {
    chars: HashMap<char, Bitmap>,
    height: i32,
}

impl FontBitmaps {
    pub fn height(&self) -> i32 { self.height }

    pub fn width(&self, s: &str) -> i32 {
        self.to_bitmaps(s).map(Bitmap::width).sum()
    }

    /// Draws string of text to the dst `Bitmap`
    /// 
    /// Returns width of drawn string in pixels
    pub fn draw_string(&self, canvas: &Bitmap, V2i { x: start_x, y }: V2i, s: &str) -> i32 {
        let mut x = start_x;
        for letter in self.to_bitmaps(s) {
            draw_bmp(canvas, letter, (x, y).into());
            x += letter.width();
        }
        x - start_x
    }

    fn to_bitmaps<'slf, 'str, 'res>(&'slf self, s: &'str str) -> impl Iterator<Item = &'res Bitmap>
        where 'slf: 'res,
              'str: 'res
    {
        s.chars().map(move |ref c| self.chars.get(c)
            .unwrap_or_else(|| panic!("No bitmap for character: {}", c)))
    }

    pub fn new(filepath: impl AsRef<Path>, height: i32) -> std::io::Result<Self> {
        use rusttype::{point, FontCollection, PositionedGlyph, Scale};

        const ALL_SYMBOLS: &str =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.,:;!?";

        let font = {
            let file = crate::file::read_all(filepath)?;
            let collection = FontCollection::from_bytes(file)
                .unwrap_or_else(|e| panic!("error constructing a FontCollection from bytes: {}", e));
            collection.into_font()
                .unwrap_or_else(|e| panic!("error turning FontCollection into a Font: {}", e))
        };

        let v_metrics_unscaled = font.v_metrics_unscaled();
        let height = height as f32
            * ((v_metrics_unscaled.ascent - v_metrics_unscaled.descent) / v_metrics_unscaled.ascent);
        let height_px = height.ceil() as i32;
        let scale = Scale::uniform(height as f32);

        // The origin of a line of text is at the baseline (roughly where
        // non-descending letters sit). We don't want to clip the text, so we shift
        // it down with an offset when laying it out. v_metrics.ascent is the
        // distance between the baseline and the highest edge of any glyph in
        // the font. That's enough to guarantee that there's no clipping.
        let v_metrics = font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent);

        let glyphs: Vec<PositionedGlyph> = font.layout(ALL_SYMBOLS, scale, offset).collect();

        let mut char_bitmaps: HashMap<char, Bitmap> = glyphs.iter().zip(ALL_SYMBOLS.chars())
            .filter_map(|(g, ch)|
                if let Some(bbox) = g.pixel_bounding_box() {
                    let width = bbox.max.x - bbox.min.x;
                    let mut bmp = Bitmap::with_dimensions(width, height_px).filled(Color::TRANSPARENT);
                    g.draw(|x, y, v| {
                        let v = utils::clamp(v, 0.0, 1.0);
                        let x = x as i32;
                        let y = y as i32 + bbox.min.y as i32;
                        bmp[(x, y)] = Color::argb(v, 1.0, 1.0, 1.0).into();
                    });

                    Some((ch, bmp))
                } else {
                    None
                }
            )
            .collect();
        let space = Bitmap::with_dimensions(height_px / 2, height_px).filled(Color::TRANSPARENT);
        char_bitmaps.insert(' ', space);

        Ok(Self {
            chars: char_bitmaps,
            height: height_px,
        })
    }
}