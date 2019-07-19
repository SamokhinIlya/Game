use std::{
    mem::{self, size_of, MaybeUninit},
    ptr,
};
use winapi::{
    shared::{
        windef::{HWND, HDC},
        ntdef::{LONG, LPCSTR},
        minwindef,
    },
    um::{
        winuser::{self, WINDOWPLACEMENT},
        wingdi::{self, BITMAPINFO},
    },
};
use crate::graphics::WindowBuffer;

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
        assert!(width > 0 && height > 0);

        use winapi::shared::windef;
        use winapi::um::{
            libloaderapi::GetModuleHandleA,
            winuser::{WNDCLASSEXA, RegisterClassExA, AdjustWindowRectEx, CreateWindowExA, GetDC},
        };

        let instance = win_assert_non_null! {
            GetModuleHandleA(ptr::null())
        };
        let class_name = {
            let class = WNDCLASSEXA {
                cbSize: size_of::<WNDCLASSEXA>() as u32,
                style: winuser::CS_HREDRAW | winuser::CS_VREDRAW, //TODO: check if CS_DBLCLKS is needed
                lpfnWndProc: Some(Self::window_class_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: instance,
                hIcon: ptr::null_mut(),   //TODO: add icon
                hCursor: ptr::null_mut(), //TODO: add cursor
                hbrBackground: ptr::null_mut(),
                lpszMenuName: ptr::null_mut(),
                lpszClassName: "main_window_class\0".as_ptr() as *const _,
                hIconSm: ptr::null_mut(), //TODO: add small icon
            };
            win_assert_non_zero! {
                RegisterClassExA(&class);
            };
            class.lpszClassName
        };
        let window_style = winuser::WS_SYSMENU | winuser::WS_CAPTION;
        let handle = {
            let mut window_dim = windef::RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            };
            win_assert_non_zero! {
                AdjustWindowRectEx(&mut window_dim, window_style | winuser::WS_VISIBLE, 0, 0);
            };
            win_assert_non_null!(
                CreateWindowExA(
                    0,
                    class_name,
                    "main_window\0".as_ptr() as *const _,
                    window_style | winuser::WS_VISIBLE,
                    winuser::CW_USEDEFAULT,
                    winuser::CW_USEDEFAULT,
                    window_dim.right - window_dim.left,
                    window_dim.bottom - window_dim.top,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    instance,
                    ptr::null_mut(),
                )
            )
        };
        let window_placement = WINDOWPLACEMENT {
            length: size_of::<WINDOWPLACEMENT>() as u32,
            ..unsafe { mem::zeroed() }
        };
        let device_context = win_assert_non_null! {
            GetDC(handle)
        };
        let bitmap_info = BITMAPINFO {
            bmiHeader: wingdi::BITMAPINFOHEADER {
                biSize: size_of::<wingdi::BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, //NOTE: negative means that bitmap is top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: wingdi::BI_RGB,
                ..unsafe { mem::zeroed() }
            },
            ..unsafe { mem::zeroed() }
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
        self.handle == unsafe { winuser::GetActiveWindow() }
    }

    pub fn toggle_fullscreen(&mut self) {
        use winapi::um::winuser::{
            MonitorFromWindow, GetMonitorInfoA,
            GetWindowLongA, SetWindowLongA,
            GetWindowPlacement, SetWindowPlacement,
            GetClientRect,
            SetWindowPos,
        };

        let current_style = win_assert_non_zero! {
            GetWindowLongA(self.handle, winuser::GWL_STYLE)
        };
        // if windowed
        if (current_style & self.windowed_style) != 0 {
            let monitor_rect = {
                let monitor = unsafe {
                    MonitorFromWindow(self.handle, winuser::MONITOR_DEFAULTTOPRIMARY)
                };
                let mut monitor_info = winuser::MONITORINFO {
                    cbSize: size_of::<winuser::MONITORINFO>() as u32,
                    ..unsafe { mem::zeroed() }
                };
                win_assert_non_zero! {
                    GetMonitorInfoA(monitor, &mut monitor_info);
                };
                monitor_info.rcMonitor
            };
            let fullscreen_width = monitor_rect.right - monitor_rect.left;
            let fullscreen_height = monitor_rect.bottom - monitor_rect.top;

            win_assert_non_zero! {
                GetWindowPlacement(self.handle, &mut self.prev_placement);
                SetWindowLongA(self.handle, winuser::GWL_STYLE, current_style & !self.windowed_style);
                SetWindowPos(
                    self.handle,
                    winuser::HWND_TOP,
                    monitor_rect.left,
                    monitor_rect.top,
                    fullscreen_width,
                    fullscreen_height,
                    winuser::SWP_NOOWNERZORDER | winuser::SWP_FRAMECHANGED,
                );
            };

            self.width = fullscreen_width;
            self.height = fullscreen_height;
        // if fullscreen
        } else {
            win_assert_non_zero! {
                SetWindowPlacement(self.handle, &self.prev_placement);
                SetWindowLongA(self.handle, winuser::GWL_STYLE, current_style | self.windowed_style);
                SetWindowPos(
                    self.handle,
                    ptr::null_mut(),
                    0,
                    0,
                    0,
                    0,
                    winuser::SWP_NOMOVE
                        | winuser::SWP_NOSIZE
                        | winuser::SWP_NOZORDER
                        | winuser::SWP_NOOWNERZORDER
                        | winuser::SWP_FRAMECHANGED,
                );
            };

            let client_rect = {
                let mut client_rect = MaybeUninit::uninit();
                win_assert_non_zero! {
                    GetClientRect(self.handle, client_rect.as_mut_ptr());
                };
                unsafe { client_rect.assume_init() }
            };

            self.width = client_rect.right;
            self.height = client_rect.bottom;
        }
    }

    pub fn blit(&self, bmp: WindowBuffer) {
        let blit_result = unsafe {
            wingdi::StretchDIBits(
                self.device_context,
                0,
                0,
                self.width,
                self.height,
                0,
                0,
                bmp.width,
                bmp.height,
                bmp.data as *mut _,
                &self.bitmap_info,
                wingdi::DIB_RGB_COLORS,
                wingdi::SRCCOPY,
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
                wingdi::DIB_RGB_COLORS,
                wingdi::SRCCOPY,
            );
        }
    }

    pub unsafe fn set_title(&self, str_buffer: &[u8]) {
        winuser::SetWindowTextA(self.handle, str_buffer.as_ptr() as LPCSTR);
    }

    unsafe extern "system" fn window_class_proc(
        window_handle: HWND,
        message: minwindef::UINT,
        w_param: minwindef::WPARAM,
        l_param: minwindef::LPARAM,
    ) -> minwindef::LRESULT {
        let mut result = 0;

        // TODO: other messages:
        //  WM_COMPACTING - system needs more memory, so we should free
        //  WM_INPUTLANGCHANGE
        match message {
            winuser::WM_CLOSE => winuser::PostQuitMessage(0),
            winuser::WM_ACTIVATEAPP => (), //TODO: pause the game and something else maybe
            _ => result = winuser::DefWindowProcA(window_handle, message, w_param, l_param),
        }

        result
    }
}

/// Message dispatch loop. Dispatches all messages in queue.
///
/// Returns `false` when WM_QUIT is received and `true` otherwise.
pub fn dispatch_messages() -> bool {
    use winuser::{PeekMessageA, TranslateMessage, DispatchMessageA};

    loop {
        let msg = unsafe {
            let mut msg = MaybeUninit::uninit();
            if PeekMessageA(msg.as_mut_ptr(), ptr::null_mut(), 0, 0, winuser::PM_REMOVE) != 0 {
                Some(msg.assume_init())
            } else {
                None
            }
        };

        match msg {
            None => break true,
            Some(msg) if msg.message == winuser::WM_QUIT => break false,
            Some(msg) => unsafe {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            },
        }
    }
}
