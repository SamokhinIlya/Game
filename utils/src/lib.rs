#![no_std]

use core::cmp::PartialOrd;

pub fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

pub fn point_clamp<T: PartialOrd>(val: (T, T), min: (T, T), max: (T, T)) -> (T, T) {
    (clamp(val.0, min.0, max.0), clamp(val.1, min.1, max.1))
}

pub fn map_range(src: f32, src_range: (f32, f32), dst_range: (f32, f32)) -> f32 {
    dst_range.0 + (src - src_range.0) / (src_range.1 - src_range.0) * (dst_range.1 - dst_range.0)
}
