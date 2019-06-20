#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Color {
    b: u8,
    g: u8,
    r: u8,
    a: u8,
}

impl From<Color> for u32 {
    fn from(c: Color) -> Self {
        unsafe { std::mem::transmute(c) }
    }
}

#[allow(dead_code)]
impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        assert!((0.0..=1.0).contains(&r), "Color::rgb: r = {} (must be from 0.0 to 1.0)", r);
        assert!((0.0..=1.0).contains(&g), "Color::rgb: g = {} (must be from 0.0 to 1.0)", g);
        assert!((0.0..=1.0).contains(&b), "Color::rgb: b = {} (must be from 0.0 to 1.0)", b);

        Self {
            a: 0xFF,
            r: (r * 255.0).round() as u8,
            g: (g * 255.0).round() as u8,
            b: (b * 255.0).round() as u8,
        }
    }

    pub fn argb(a: f32, r: f32, g: f32, b: f32) -> Self {
        assert!((0.0..=1.0).contains(&a), "Color::rgb: a = {} (must be from 0.0 to 1.0)", a);

        let mut color = Self::rgb(r, g, b);
        color.a = (a * 255.0).round() as u8;
        color
    }

    pub const A_MASK: u32 = 0xFF00_0000;
    pub const R_MASK: u32 = 0x00FF_0000;
    pub const G_MASK: u32 = 0x0000_FF00;
    pub const B_MASK: u32 = 0x0000_00FF;

    pub const TRANSPARENT: Self = Self {
        b: 0x00,
        g: 0x00,
        r: 0x00,
        a: 0x00,
    };
    pub const BLACK: Self = Self {
        b: 0x00,
        g: 0x00,
        r: 0x00,
        a: 0xFF,
    };
    pub const WHITE: Self = Self {
        b: 0xFF,
        g: 0xFF,
        r: 0xFF,
        a: 0xFF,
    };
    pub const YELLOW: Self = Self {
        b: 0x00,
        g: 0xFF,
        r: 0xFF,
        a: 0xFF,
    };
    pub const RED: Self = Self {
        b: 0x00,
        g: 0x00,
        r: 0xFF,
        a: 0xFF,
    };
    pub const PURPLE: Self = Self {
        b: 0xFF,
        g: 0x00,
        r: 0xFF,
        a: 0xFF,
    };
    pub const GREY: Self = Self {
        b: 0xFF / 2,
        g: 0xFF / 2,
        r: 0xFF / 2,
        a: 0xFF,
    };
}
