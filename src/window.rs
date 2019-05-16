use core::{
    mem::{self, size_of},
    ptr,
};
use platform::{
    win_assert_non_null,
    win_assert_non_zero,
    debug,
    graphics::WindowBuffer,
};
use winapi::{
    ctypes::*,
    shared::{minwindef::*, windef::*},
    um::{winnt::*, wingdi::*, winuser::*},
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

        let instance = win_assert_non_null!( GetModuleHandleA(ptr::null()) );
        let class = WNDCLASSEXA {
            cbSize: size_of::<WNDCLASSEXA>() as u32,
            style: CS_HREDRAW | CS_VREDRAW, //TODO: check if CS_DBLCLKS is needed
            lpfnWndProc: Some(window_class_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: ptr::null_mut(),   //TODO: add icon
            hCursor: ptr::null_mut(), //TODO: add cursor
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null_mut(),
            lpszClassName: "main_window_class\0".as_ptr() as *const c_char,
            hIconSm: ptr::null_mut(), //TODO: add small icon
        };
        win_assert_non_zero!( RegisterClassExA(&class) );

        let window_style = WS_SYSMENU | WS_CAPTION;
        let mut window_dim = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        win_assert_non_zero!(
            AdjustWindowRectEx(
                &mut window_dim, 
                window_style | WS_VISIBLE, 
                0, 
                0,
            )
        );
        let handle = win_assert_non_null!(
            CreateWindowExA(
                0,
                class.lpszClassName,
                "main_window\0".as_ptr() as *const c_char,
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
        );

        let window_placement = WINDOWPLACEMENT {
            length: size_of::<WINDOWPLACEMENT>() as u32,
            ..unsafe { mem::zeroed() }
        };

        let device_context = win_assert_non_null!( GetDC(handle) );

        let bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, //NOTE: negative means that bitmap is top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..unsafe { mem::zeroed() }
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

    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }
    pub fn handle(&self) -> HWND { self.handle }

    pub fn is_active(&self) -> bool {
        self.handle == unsafe { GetActiveWindow() }
    }

    pub fn toggle_fullscreen(&mut self) {
        let current_style = win_assert_non_zero!( GetWindowLongA(self.handle, GWL_STYLE) );
        // if windowed
        if (current_style & self.windowed_style) != 0 {
            let monitor_info = {
                let monitor = unsafe { MonitorFromWindow(self.handle, MONITOR_DEFAULTTOPRIMARY) };
                let mut mon_info = MONITORINFO {
                    cbSize: size_of::<MONITORINFO>() as u32,
                    ..unsafe { mem::zeroed() }
                };
                win_assert_non_zero!( GetMonitorInfoA(monitor, &mut mon_info) );

                mon_info
            };
            let fullscreen_window_width = monitor_info.rcMonitor.right - monitor_info.rcMonitor.left;
            let fullscreen_window_height = monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top;

            win_assert_non_zero!( GetWindowPlacement(self.handle, &mut self.prev_placement) );
            win_assert_non_zero!( SetWindowLongA(self.handle, GWL_STYLE, current_style & !self.windowed_style) );
            win_assert_non_zero!(
                SetWindowPos(
                    self.handle,
                    HWND_TOP,
                    monitor_info.rcMonitor.left,
                    monitor_info.rcMonitor.top,
                    fullscreen_window_width,
                    fullscreen_window_height,
                    SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
                )
            );

            self.width = fullscreen_window_width;
            self.height = fullscreen_window_height;
        // if fullscreen
        } else {
            win_assert_non_zero!( SetWindowPlacement(self.handle, &self.prev_placement) );
            win_assert_non_zero!( SetWindowLongA(self.handle, GWL_STYLE, current_style | self.windowed_style) );
            win_assert_non_zero!(
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
            );
            let mut client_rect = unsafe { mem::uninitialized() };
            win_assert_non_zero!( GetClientRect(self.handle, &mut client_rect) );

            self.width = client_rect.right;
            self.height = client_rect.bottom;
        }
    }

    pub fn blit(&self, bmp: WindowBuffer) {
        let blit_result = unsafe {
            StretchDIBits(
                self.device_context,
                0,
                0,
                self.width,
                self.height,
                0,
                0,
                bmp.width,
                bmp.height,
                bmp.data as *mut c_void,
                &self.bitmap_info,
                DIB_RGB_COLORS,
                SRCCOPY,
            )
        };
        if blit_result == 0 {
            panic!(
                "StretchDIBits in Window::blit(...) failed.
                StretchDIBits {{
                    hdc: {:p},
                    xDest: {},
                    yDest: {},
                    DestWidth: {},
                    DestHeight: {},
                    xSrc: {},
                    ySrc: {},
                    SrcWidth: {},
                    SrcHeight: {},
                    lpBits: {:p},
                    lpbmi: {:p},
                    iUsage: {},
                    rop: {},
                }}",
                self.device_context,
                0,
                0,
                self.width,
                self.height,
                0,
                0,
                bmp.width,
                bmp.height,
                bmp.data,
                &self.bitmap_info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
        }
    }

    pub fn set_title(&self, str_buffer: &[u8]) {
        unsafe { SetWindowTextA(self.handle, str_buffer.as_ptr() as LPCSTR) };
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

    let mut result = 0;
    match message {
        WM_CLOSE       => PostQuitMessage(0),
        WM_ACTIVATEAPP => (), //TODO: pause the game and something else maybe
        _              => result = DefWindowProcA(window_handle, message, w_param, l_param),
    }

    result
}