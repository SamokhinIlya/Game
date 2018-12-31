use core::{mem, ptr};
use platform::*;
use platform_internal::*;
use winapi::{ctypes::*, um::winuser::*};

/* TODO:
 - controller support
 - provide game a way to change window and rendering resolution
 - audio
 - logging
*/

fn main() {
    time::init();
    let mut window = window::Window::with_dimensions(1920 / 2, 1080 / 2);
    let mut window_bmp = graphics::Bitmap::with_dimensions(window.width(), window.height());
    let mut input: input::Input = Default::default();
    let game_data_ptr = game::startup(window_bmp.width(), window_bmp.height());
   
    let mut running = true;
    let mut went_inactive = false;
    while running {
        let frame_counter = time::Counter::start();

        use platform::input::KBKey;
        running = process_messages();
        {
            use platform::input::MouseKey;
            if window.is_active() {
                went_inactive = false;
                for &key in input::KBKey::variants() {
                    let key_state = unsafe { GetAsyncKeyState(key as c_int) };
                    let is_down = key_state < 0;
                    input.keyboard[key].update(is_down);
                }

                let mut mouse_point = unsafe { mem::uninitialized() };
                win_assert_non_zero!( GetCursorPos(&mut mouse_point) );
                win_assert_non_zero!( ScreenToClient(window.handle(), &mut mouse_point) );

                input.mouse.x = mouse_point.x;
                input.mouse.y = mouse_point.y;
                input.mouse[MouseKey::LB].update(unsafe { GetAsyncKeyState(VK_LBUTTON) } < 0);
                input.mouse[MouseKey::RB].update(unsafe { GetAsyncKeyState(VK_RBUTTON) } < 0);
                input.mouse[MouseKey::MB].update(unsafe { GetAsyncKeyState(VK_MBUTTON) } < 0);
            } else if !went_inactive {
                went_inactive = true;
                for &key in input::KBKey::variants() {
                    input.keyboard[key].update(false);
                }
                input.mouse[MouseKey::LB].update(false);
                input.mouse[MouseKey::RB].update(false);
                input.mouse[MouseKey::MB].update(false);
            }
        }

        if (input.keyboard[KBKey::Alt].is_down() && input.keyboard[KBKey::Enter].pressed())
            || input.keyboard[KBKey::F11].pressed()
        {
            window.toggle_fullscreen();
        }

        let game_update_counter = time::Counter::start();
        let game_info: String = game::update_and_render(&mut window_bmp, &input, game_data_ptr);
        let game_update_ms_elapsed = game_update_counter.end().as_ms();

        window.blit(&window_bmp);

        //TODO: wait a bit
        let mut frame_ticks_elapsed = frame_counter.elapsed();
        let mut frame_ms_elapsed = frame_ticks_elapsed.as_ms();
        while frame_ms_elapsed < 16.0 {
            frame_ticks_elapsed = frame_counter.elapsed();
            frame_ms_elapsed = frame_ticks_elapsed.as_ms();
        } 
        input.dt = frame_ticks_elapsed.as_secs() as f32;

        let mut str_buffer: [u8; 256] = unsafe { mem::uninitialized() };
        use std::io::Write;
        write!(
            &mut str_buffer as &mut [u8],
            "frame: {:>3.3} ms, game_update: {:>3.3} ms, {:>2.2} fps, dt: {} sec{}\0",
            frame_ms_elapsed, game_update_ms_elapsed, frame_ticks_elapsed.as_secs().recip(), input.dt, game_info,
        ).unwrap();
        window.set_title(&str_buffer);
    }
}

fn process_messages() -> bool {
    loop {
        let mut msg = unsafe { mem::uninitialized() };
        let peek_msg_result = unsafe { PeekMessageA(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) };
        let message_available = peek_msg_result != 0;
        if !message_available {
            break true;
        }
        if msg.message == WM_QUIT {
            break false;
        }
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
}
