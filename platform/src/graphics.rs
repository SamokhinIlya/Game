use winapi::ctypes::c_void;

#[derive(Copy, Clone)]
pub struct WindowBuffer {
    pub data: *mut c_void,
    pub width: i32,
    pub height: i32,
}

impl WindowBuffer {
    pub fn with_dimensions(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0);

        let data = {
            let mut vec = Vec::<u32>::with_capacity(width as usize * height as usize);
            let ptr = vec.as_mut_ptr();
            core::mem::forget(vec);
            ptr as *mut c_void
        };

        Self { data, width, height }
    }
}