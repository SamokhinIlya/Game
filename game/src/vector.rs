use core::{
    ops::{Add, Sub, Mul},
};

#[derive(Copy, Clone, Default, Debug)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

macro_rules! v2 {
    ($x:expr, $y:expr$(,)*) => {
        V2 { x: $x, y: $y }
    };
}

impl V2 {
    pub fn new() -> Self {
        v2!(0.0, 0.0)
    }
}

impl Add for V2 {
    type Output = V2;
    fn add(self, rhs: V2) -> V2 {
        v2!(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for V2 {
    type Output = V2;
    fn sub(self, rhs: V2) -> V2 {
        v2!(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for V2 {
    type Output = V2;
    fn mul(self, scalar: f32) -> V2 {
        v2!(self.x * scalar, self.y * scalar)
    }
}

impl Mul<V2> for f32 {
    type Output = V2;
    fn mul(self, v: V2) -> V2 {
        v * self
    }
}

pub fn dot(lhs: V2, rhs: V2) -> f32 {
    lhs.x * rhs.x + lhs.y * rhs.y
}

pub fn distance_sq(lhs: V2, rhs: V2) -> f32 {
    let distance_vector = rhs - lhs;
    dot(distance_vector, distance_vector)
}

use std::fmt;
impl fmt::Display for V2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "V2({:.2}, {:.2})", self.x, self.y)
    }
}