pub use core::{
    fmt::Debug,
    ops::{
        Add, AddAssign,
        Sub, SubAssign,
        Mul, MulAssign,
        Div, DivAssign,
        Neg,
    },
};

pub trait Num32:
    Sized + Copy + Clone
    + Send + Sync
    + Debug
    + PartialEq + PartialOrd
    + Add<Output = Self> + AddAssign
    + Sub<Output = Self> + SubAssign
    + Mul<Output = Self> + MulAssign
    + Div<Output = Self> + DivAssign
    + Neg<Output = Self>
{}

impl Num32 for i32 {}
impl Num32 for f32 {}
