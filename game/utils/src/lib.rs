extern crate core;

use core::cmp::PartialOrd;

#[inline]
pub fn clamp<T>(val: &mut T, min: T, max: T)
    where T: PartialOrd
{
    if *val < min {
        *val = min
    } else if *val > max {
        *val = max
    }
}

#[inline]
pub fn map_range(src: f32, src_range: (f32, f32), dst_range: (f32, f32)) -> f32 {
    dst_range.0 + (src - src_range.0) / (src_range.1 - src_range.0) * (dst_range.1 - dst_range.0)
}