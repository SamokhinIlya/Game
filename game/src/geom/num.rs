pub use core::{
    fmt::Debug,
    ops::{
        Add, AddAssign,
        Sub, SubAssign,
        Mul, MulAssign,
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
    + Neg<Output = Self>
{}

impl Num32 for i32 {}
impl Num32 for f32 {}
