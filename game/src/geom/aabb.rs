use super::{
    num::Num32,
    vector::V2,
};

/// Axis-aligned bounding box
#[derive(Copy, Clone, Debug)]
pub struct AABB<T: Num32> {
    pub min: V2<T>,
    pub max: V2<T>,
}

#[allow(dead_code)]
impl<T: Num32> AABB<T> {
    pub fn right(self)  -> T { self.max.x }
    pub fn left(self)   -> T { self.min.x }
    pub fn top(self)    -> T { self.max.y }
    pub fn bottom(self) -> T { self.min.y }

    pub fn top_left(self)     -> V2<T> { (self.min.x, self.max.y).into() }
    pub fn top_right(self)    -> V2<T> { self.max }
    pub fn bottom_left(self)  -> V2<T> { self.min }
    pub fn bottom_right(self) -> V2<T> { (self.max.x, self.min.y).into() }

    pub fn translate(self, by: V2<T>) -> Self {
        Self {
            min: self.min + by,
            max: self.max + by,
        }
    }
}