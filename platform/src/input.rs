use core::{
    default::Default,
    ops::{Index, IndexMut},
};

#[derive(Default)]
pub struct Input {
    pub keyboard: KeyboardState,
    pub mouse: MouseState,
    pub dt: f32,
}

pub struct KeyboardState {
    keys: [DigitalKey; 0xFF],
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self { keys: [Default::default(); 0xFF] }
    }
}

impl Index<KBKey> for KeyboardState {
    type Output = DigitalKey;
    fn index(&self, key: KBKey) -> &Self::Output {
        &self.keys[key as usize]
    }
}

impl IndexMut<KBKey> for KeyboardState {
    fn index_mut(&mut self, key: KBKey) -> &mut DigitalKey {
        &mut self.keys[key as usize]
    }
}

//TODO: scroll wheel
#[derive(Default)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    keys: [DigitalKey; 3],
}

impl MouseState {
    #[inline(always)] pub fn pos(&self) -> (i32, i32) { (self.x, self.y) }
}

//TODO: other buttons
pub enum MouseKey {
    LB = 0,
    RB = 1,
    MB = 2,
}

impl Index<MouseKey> for MouseState {
    type Output = DigitalKey;
    fn index(&self, key: MouseKey) -> &Self::Output {
        &self.keys[key as usize]
    }
}

impl IndexMut<MouseKey> for MouseState {
    fn index_mut(&mut self, key: MouseKey) -> &mut DigitalKey {
        &mut self.keys[key as usize]
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct DigitalKey {
    curr: bool,
    prev: bool,
}

#[allow(dead_code)]
impl DigitalKey {
    #[inline(always)] pub fn is_down(self) -> bool { self.curr }
    #[inline(always)] pub fn is_up(self) -> bool { !self.curr }

    #[inline(always)] pub fn pressed(self) -> bool { self.curr && !self.prev }
    #[inline(always)] pub fn released(self) -> bool { !self.curr && self.prev }

    #[inline(always)]
    pub fn update(&mut self, new: bool) {
        self.prev = self.curr;
        self.curr = new;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum KBKey {
    Backspace = 0x08,
    Tab = 0x09,

    Enter = 0x0D,

    Shift = 0x10,
    Ctrl = 0x11,
    Alt = 0x12,

    CapsLock = 0x14,

    Escape = 0x1B,

    Space = 0x20,
    PageUp = 0x21,
    PageDown = 0x22,
    End = 0x23,
    Home = 0x24,
    Left = 0x25,
    Up = 0x26,
    Right = 0x27,
    Down = 0x28,

    PrintScreen = 0x2C,
    Insert = 0x2D,
    Delete = 0x2E,

    D0 = 0x30,
    D1 = 0x31,
    D2 = 0x32,
    D3 = 0x33,
    D4 = 0x34,
    D5 = 0x35,
    D6 = 0x36,
    D7 = 0x37,
    D8 = 0x38,
    D9 = 0x39,

    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,

    N0 = 0x60,
    N1 = 0x61,
    N2 = 0x62,
    N3 = 0x63,
    N4 = 0x64,
    N5 = 0x65,
    N6 = 0x66,
    N7 = 0x67,
    N8 = 0x68,
    N9 = 0x69,
    NMul = 0x6A,
    NAdd = 0x6B,

    NSub = 0x6D,
    NDec = 0x6E,
    NDiv = 0x6F,
    F1 = 0x70,
    F2 = 0x71,
    F3 = 0x72,
    F4 = 0x73,
    F5 = 0x74,
    F6 = 0x75,
    F7 = 0x76,
    F8 = 0x77,
    F9 = 0x78,
    F10 = 0x79,
    F11 = 0x7A,
    F12 = 0x7B,
    F13 = 0x7C,
    F14 = 0x7D,
    F15 = 0x7E,
    F16 = 0x7F,
    F17 = 0x80,
    F18 = 0x81,
    F19 = 0x82,
    F20 = 0x83,
    F21 = 0x84,
    F22 = 0x85,
    F23 = 0x86,
    F24 = 0x87,

    LShift = 0xA0,
    Rshift = 0xA1,
    LCtrl = 0xA2,
    RCtrl = 0xA3,
    LAlt = 0xA4,
    RAlt = 0xA5,

    Semicolon = 0xBA,
    Plus = 0xBB,
    Comma = 0xBC,
    Minus = 0xBD,
    Period = 0xBE,
    Slash = 0xBF,
    Tilda = 0xC0,

    LBracket = 0xDB,
    Backslash = 0xDC,
    RBracket = 0xDD,
    Quote = 0xDE,
}

impl KBKey {
    pub fn variants() -> core::slice::Iter<'static, Self> {
        use self::KBKey::*;
        const VARIANTS: [KBKey; 112] = [
            Backspace,
            Tab,
            Enter,
            Shift,
            Ctrl,
            Alt,
            CapsLock,
            Escape,
            Space,
            PageUp,
            PageDown,
            End,
            Home,
            Left,
            Up,
            Right,
            Down,
            PrintScreen,
            Insert,
            Delete,
            D0,
            D1,
            D2,
            D3,
            D4,
            D5,
            D6,
            D7,
            D8,
            D9,
            A,
            B,
            C,
            D,
            E,
            F,
            G,
            H,
            I,
            J,
            K,
            L,
            M,
            N,
            O,
            P,
            Q,
            R,
            S,
            T,
            U,
            V,
            W,
            X,
            Y,
            Z,
            N0,
            N1,
            N2,
            N3,
            N4,
            N5,
            N6,
            N7,
            N8,
            N9,
            NMul,
            NAdd,
            NSub,
            NDec,
            NDiv,
            F1,
            F2,
            F3,
            F4,
            F5,
            F6,
            F7,
            F8,
            F9,
            F10,
            F11,
            F12,
            F13,
            F14,
            F15,
            F16,
            F17,
            F18,
            F19,
            F20,
            F21,
            F22,
            F23,
            F24,
            LShift,
            Rshift,
            LCtrl,
            RCtrl,
            LAlt,
            RAlt,
            Semicolon,
            Plus,
            Comma,
            Minus,
            Period,
            Slash,
            Tilda,
            LBracket,
            Backslash,
            RBracket,
            Quote,
        ];

        VARIANTS.iter()
    }
}