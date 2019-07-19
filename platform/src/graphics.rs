#[derive(Copy, Clone)]
pub struct WindowBuffer {
    pub data: *mut u32,
    pub width: i32,
    pub height: i32,
}

impl WindowBuffer {
    pub fn with_dimensions(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0);

        use std::alloc::{alloc, Layout};
        use std::mem::{size_of, align_of};

        let data = unsafe {
            alloc(
                Layout::from_size_align_unchecked(
                    width as usize * height as usize * size_of::<u32>(),
                    align_of::<u32>(),
                )
            ) as *mut u32
        };

        Self { data, width, height }
    }
}