use core::{
    mem,
    mem::size_of,
    ptr,
    ops::Drop,
};
use winapi::{
    ctypes::*,
    shared::{minwindef::*, windef::*},
    um::winnt::*,
    um::{wingdi::*, winuser::*},
};
use platform::{
    debug,
    graphics::Bitmap,
};

pub struct Window {
    handle: HWND,
    width: i32,
    height: i32,
    prev_placement: WINDOWPLACEMENT,
    windowed_style: LONG,
    device_context: HDC,
    bitmap_info: BITMAPINFO,
}

impl Window {
    pub fn with_dimensions(width: i32, height: i32) -> Self {
        use winapi::um::libloaderapi::GetModuleHandleA;

        let instance = unsafe { GetModuleHandleA(ptr::null()) };
        if instance.is_null() {
            debug::panic_with_last_error_message("GetModuleHandleA");
        }

        let class_name = "main_window_class\0";
        let class = WNDCLASSEXA {
            cbSize: size_of::<WNDCLASSEXA>() as u32,
            style: CS_HREDRAW | CS_VREDRAW, //TODO: check if CS_DBLCLKS is needed
            lpfnWndProc: Some(window_class_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: ptr::null_mut(),   //TODO: add icon
            hCursor: ptr::null_mut(), //TODO: check if this works and add cursor later maybe
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null_mut(),
            lpszClassName: class_name.as_ptr() as *const c_char,
            hIconSm: ptr::null_mut(), //TODO: add small icon
        };
        if unsafe { RegisterClassExA(&class) } == 0 {
            debug::panic_with_last_error_message("RegisterClassExA");
        }

        let window_name = "main_window\0";
        let window_style = WS_SYSMENU | WS_CAPTION;
        let mut window_dim = winapi::shared::windef::RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        let adjust_window_rect_result = unsafe {
            AdjustWindowRectEx(
                &mut window_dim, 
                window_style | WS_VISIBLE, 
                0, 
                0,
            )
        };
        if adjust_window_rect_result == 0 {
            debug::panic_with_last_error_message("AdjustWindowRectEx");
        }
        let handle = unsafe {
            CreateWindowExA(
                0,
                class_name.as_ptr() as *const c_char,
                window_name.as_ptr() as *const c_char,
                window_style | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                window_dim.right - window_dim.left,
                window_dim.bottom - window_dim.top,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null_mut(),
            )
        };
        if handle.is_null() {
            debug::panic_with_last_error_message("CreateWindowExA");
        }

        let window_placement = WINDOWPLACEMENT {
            length: size_of::<WINDOWPLACEMENT>() as u32,
            ..unsafe { mem::zeroed() }
        };

        let device_context = unsafe { GetDC(handle) };
        if device_context.is_null() {
            //TODO: proper error handling
            debug::panic_with_last_error_message("GetDC");
        }

        let bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, //NOTE: negative value suggests that bitmap is top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: unsafe { mem::zeroed() },
        };

        Self {
            handle,
            width,
            height,
            prev_placement: window_placement,
            windowed_style: window_style as LONG,
            device_context,
            bitmap_info,
        }
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.handle == unsafe { GetActiveWindow() }
    }

    #[inline]
    pub fn width(&self) -> i32 { self.width }

    #[inline]
    pub fn height(&self) -> i32 { self.height }

    #[inline]
    pub fn handle(&self) -> HWND { self.handle }

    pub fn toggle_fullscreen(&mut self) {
        let current_style = unsafe { GetWindowLongA(self.handle, GWL_STYLE) };
        if current_style == 0 {
            debug::panic_with_last_error_message("GetWindowLongA");
        }
        //NOTE: if windowed
        if (current_style & self.windowed_style) != 0 {
            let mut monitor_info = MONITORINFO {
                cbSize: size_of::<MONITORINFO>() as u32,
                ..unsafe { mem::zeroed() }
            };

            let get_window_placement_result =
                unsafe { GetWindowPlacement(self.handle, &mut self.prev_placement) };
            if get_window_placement_result == 0 {
                debug::panic_with_last_error_message("GetWindowPlacement");
            }

            let monitor = unsafe { MonitorFromWindow(self.handle, MONITOR_DEFAULTTOPRIMARY) };

            let get_monitor_info_result =
                unsafe { GetMonitorInfoA(monitor, &mut monitor_info) };
            if get_monitor_info_result == 0 {
                debug::panic_with_last_error_message("GetMonitorInfoA");
            }

            if unsafe {
                SetWindowLongA(self.handle, GWL_STYLE, current_style & !self.windowed_style)
            } == 0
            {
                debug::panic_with_last_error_message("SetWindowLongA");
            }
            let fullscreen_window_width =
                monitor_info.rcMonitor.right - monitor_info.rcMonitor.left;
            let fullscreen_window_height =
                monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top;
            let set_window_pos_result = unsafe {
                SetWindowPos(
                    self.handle,
                    HWND_TOP,
                    monitor_info.rcMonitor.left,
                    monitor_info.rcMonitor.top,
                    fullscreen_window_width,
                    fullscreen_window_height,
                    SWP_NOOWNERZORDER | SWP_FRAMECHANGED, //TODO: check other options and NOOWNERZORDER
                )
            };
            if set_window_pos_result == 0 {
                debug::panic_with_last_error_message("SetWindowPos");
            }

            self.width = fullscreen_window_width;
            self.height = fullscreen_window_height;
        } else {
            //NOTE: if fullscreen
            if unsafe {
                SetWindowLongA(self.handle, GWL_STYLE, current_style | self.windowed_style)
            } == 0
            {
                debug::panic_with_last_error_message("SetWindowLongA");
            }

            if unsafe { SetWindowPlacement(self.handle, &self.prev_placement) } == 0 {
                debug::panic_with_last_error_message("SetWindowPlacement");
            }

            let set_window_pos_result = unsafe {
                SetWindowPos(
                    self.handle,
                    ptr::null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE
                        | SWP_NOSIZE
                        | SWP_NOZORDER
                        | SWP_NOOWNERZORDER
                        | SWP_FRAMECHANGED,
                )
            };
            if set_window_pos_result == 0 {
                debug::panic_with_last_error_message("SetWindowPos");
            }

            let mut client_rect = unsafe { mem::uninitialized() };
            if unsafe { GetClientRect(self.handle, &mut client_rect) } == 0 {
                debug::panic_with_last_error_message("GetClientRect");
            }

            self.width = client_rect.right;
            self.height = client_rect.bottom;
        }
    }

    pub fn blit(&self, bmp: &Bitmap) {
        let blit_result = unsafe {
            StretchDIBits(
                self.device_context,
                0,
                0,
                self.width,
                self.height,
                0,
                0,
                bmp.width(),
                bmp.height(),
                bmp.data() as *const c_void,
                &self.bitmap_info,
                DIB_RGB_COLORS,
                SRCCOPY,
            )
        };
        if blit_result == 0 {
            panic!("StretchDIBits in Window::blit(...) failed");
        }
    }

    pub fn set_title(&self, str_buffer: &[u8]) {
        unsafe { SetWindowTextA(self.handle, str_buffer.as_ptr() as LPCSTR) };
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if unsafe { DestroyWindow(self.handle) } == 0 {
            debug::panic_with_last_error_message("DestroyWindow");
        }
    }
}

unsafe extern "system" fn window_class_proc(
    window_handle: HWND,
    message: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    /*TODO: other messages:
        WM_COMPACTING - system needs more memory, so we should free
        WM_INPUTLANGCHANGE 
    */
    match message {
        WM_CLOSE => {
            PostQuitMessage(0);
            0
        }
        WM_ACTIVATEAPP => {
            //TODO: pause the game and something else maybe
            0
        }
        _ => DefWindowProcA(window_handle, message, w_param, l_param),
    }
}