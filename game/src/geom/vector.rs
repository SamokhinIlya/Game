use core::ops::{
    Add, AddAssign,
    Sub, SubAssign,
    Mul,
    Neg,
};
pub use super::num::Num32;

pub mod prelude {
    pub use super::{
        Num32,
        V2, V2i, V2f,
    };
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct V2<T: Num32> {
    pub x: T,
    pub y: T,
}

pub type V2i = V2<i32>;
pub type V2f = V2<f32>;

//TODO: add const when
// "trait bounds other than `Sized` on const fn parameters are unstable"
// is no more
impl<T: Num32> V2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn diag(val: T) -> Self {
        Self { x: val, y: val }
    }

    pub fn map(self, f: impl Fn(T) -> T) -> Self {
        Self { x: f(self.x), y: f(self.y) }
    }
}

impl V2<f32> {
    pub fn round(self) -> Self {
        Self::map(self, f32::round)
    }

    pub fn floor(self) -> Self {
        Self::map(self, f32::floor)
    }

    pub fn ceil(self) -> Self {
        Self::map(self, f32::ceil)
    }

    pub fn trunc(self) -> Self {
        Self::map(self, f32::trunc)
    }
}

impl From<V2i> for V2f {
    fn from(v: V2i) -> Self {
        Self { x: v.x as f32, y: v.y as f32 }
    }
}

impl From<V2f> for V2i {
    fn from(v: V2f) -> Self {
        Self { x: v.x as i32, y: v.y as i32 }
    }
}

impl<T: Num32> From<(T, T)> for V2<T> {
    fn from((x, y): (T, T)) -> Self {
        Self { x, y }
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
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
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
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
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
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

impl Mul<V2<i32>> for i32 {
    type Output = V2<i32>;
    fn mul(self, rhs: Self::Output) -> Self::Output {
        Self::Output { x: self * rhs.x, y: self * rhs.y }
    }
}

impl Mul<V2<f32>> for f32 {
    type Output = V2<f32>;
    fn mul(self, rhs: Self::Output) -> Self::Output {
        Self::Output { x: self * rhs.x, y: self * rhs.y }
    }
}

impl<T: Num32> Neg for V2<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
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