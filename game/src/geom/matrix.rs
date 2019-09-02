use super::num::*;
use super::vector::V2;

pub struct Mat2<T: Num32>(pub V2<T>, pub V2<T>);

impl<T: Num32> From<[[T; 2]; 2]> for Mat2<T> {
    fn from([[x0, x1], [y0, y1]]: [[T; 2]; 2]) -> Self {
        Self(V2 { x: x0, y: y0 }, V2 { x: x1, y: y1 })
    }
}

impl<T: Num32> Mul<V2<T>> for &Mat2<T> {
    type Output = V2<T>;
    fn mul(self, v: Self::Output) -> Self::Output {
        self.0 * v.x + self.1 * v.y
    }
}