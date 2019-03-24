use core::fmt::Debug;
use core::ops::{
    Add, AddAssign,
    Sub, SubAssign,
    Mul, MulAssign,
    Neg,
};

pub mod prelude {
    pub use super::{
        Num32,
        V2, V2i, V2f,
    };
}

pub trait Num32:
    Sized + Copy + Clone
    + Send + Sync
    + Debug
    + PartialEq
    + Add<Output = Self> + AddAssign
    + Sub<Output = Self> + SubAssign
    + Mul<Output = Self> + MulAssign
    + Neg<Output = Self>
{}

impl Num32 for i32 {}
impl Num32 for f32 {}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct V2<T: Num32> {
    pub x: T,
    pub y: T,
}

pub type V2i = V2<i32>;
pub type V2f = V2<f32>;

macro_rules! v2 {
    ($x:expr, $y:expr$(,)*) => {
        V2 { x: $x, y: $y }
    };
    ($val:expr) => {
        V2 { x: $val, y: $val }
    };
}

//TODO: add const when
// "trait bounds other than `Sized` on const fn parameters are unstable"
// is no more
impl<T: Num32> V2<T> {
    pub fn new(x: T, y: T) -> Self {
        V2 { x, y }
    }

    pub fn diag(val: T) -> Self {
        Self::new(val, val)
    }

    pub fn map<F>(self, f: F) -> Self
        where F: Fn(T) -> T
    {
        Self::new(f(self.x), f(self.y))
    }
}

impl V2<f32> {
    pub fn floor(self) -> Self {
        Self::map(self, f32::floor)
    }

    pub fn trunc(self) -> Self {
        Self::map(self, f32::trunc)
    }
}

impl From<V2i> for V2f {
    fn from(v: V2i) -> V2f {
        v2!(v.x as f32, v.y as f32)
    }
}

impl From<V2f> for V2i {
    fn from(v: V2f) -> V2i {
        v2!(v.x as i32, v.y as i32)
    }
}

impl<T: Num32> From<(T, T)> for V2<T> {
    fn from((x, y): (T, T)) -> Self {
        v2!(x, y)
    }
}

impl<T: Num32> Into<(T, T)> for V2<T> {
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T: Num32> Add for V2<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        v2!(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T: Num32> AddAssign for V2<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: Num32> Sub for V2<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        V2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: Num32> SubAssign for V2<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: Num32> Mul<T> for V2<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        V2::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<V2<i32>> for i32 {
    type Output = V2<i32>;
    fn mul(self, rhs: Self::Output) -> Self::Output {
        V2::new(self * rhs.x, self * rhs.y)
    }
}

impl Mul<V2<f32>> for f32 {
    type Output = V2<f32>;
    fn mul(self, rhs: Self::Output) -> Self::Output {
        V2::new(self * rhs.x, self * rhs.y)
    }
}

impl<T: Num32> Neg for V2<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        V2::new(-self.x, -self.y)
    }
}

pub fn dot<T: Num32>(lhs: V2<T>, rhs: V2<T>) -> T {
    lhs.x * rhs.x + lhs.y * rhs.y
}

pub fn distance_sq<T: Num32>(lhs: V2<T>, rhs: V2<T>) -> T {
    let distance_vector = rhs - lhs;
    dot(distance_vector, distance_vector)
}

use std::fmt;
impl fmt::Display for V2f {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "V2({:.2}, {:.2})", self.x, self.y)
    }
}