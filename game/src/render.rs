use utils::clamp;
use crate::bitmap::Bitmap;

pub fn fill_rect(dst_bmp: &Bitmap, p0: (i32, i32), p1: (i32, i32), color: Color) {
    for row in dst_bmp.clamped_view(p0, p1) {
        for pxl in row {
            *pxl = color.into();
        }
    }
}

pub fn draw_bmp(dst_bmp: &Bitmap, src_bmp: &Bitmap, p: (i32, i32)) {
    let src0 = (
        if p.0 < 0 { -p.0 } else { 0 },
        if p.1 < 0 { -p.1 } else { 0 }
    );
    let src1 = (src_bmp.width(), src_bmp.height());

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

pub fn render_font(font_data: &'static [u8]) -> Bitmap {
    use rusttype::{point, FontCollection, PositionedGlyph, Scale};

    let font = {
        let collection = FontCollection::from_bytes(font_data).unwrap_or_else(|e| {
            panic!("error constructing a FontCollection from bytes: {}", e);
        });

        collection.into_font().unwrap_or_else(|e| {
            panic!("error turning FontCollection into a Font: {}", e);
        })
    };

    let height: f32 = 20.0;
    let pixel_height = height.ceil() as usize;

    let scale = Scale::uniform(height);

    // The origin of a line of text is at the baseline (roughly where
    // non-descending letters sit). We don't want to clip the text, so we shift
    // it down with an offset when laying it out. v_metrics.ascent is the
    // distance between the baseline and the highest edge of any glyph in
    // the font. That's enough to guarantee that there's no clipping.
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    const ALL_SYMBOLS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGIJKLMNOPQRSTUVWXYZ0123456789-.,!?";
    let glyphs: Vec<PositionedGlyph> = font.layout(ALL_SYMBOLS, scale, offset).collect();

    // Find the most visually pleasing width to display
    let width = glyphs
        .iter()
        .rev()
        .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
        .next()
        .unwrap_or(0.0)
        .ceil() as usize;

    println!("width: {}, height: {}", width, pixel_height);

    let mut font_bmp = Bitmap::with_dimensions(width as i32, pixel_height as i32).filled(Color::BLACK);
    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let color = if v > 0.5 { Color::WHITE } else { Color::BLACK };
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;
                // There's still a possibility that the glyph clips the boundaries of the bitmap
                if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                    let x = x as usize;
                    let y = y as usize;
                    font_bmp[(x, y)] = color.into();
                }
            })
        }
    }

    font_bmp
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
