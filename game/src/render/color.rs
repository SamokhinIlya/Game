use utils::clamp;

#[derive(Copy, Clone, PartialEq, Eq)]
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
        let data = Self::A_MASK
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
