use platform::*;

/* TODO: what is missing
 - controller support
 - provide game a way to change window and rendering resolution
 - audio
 - logging
*/

fn main() {
    let mut window = window::Window::with_dimensions(1920 / 2, 1080 / 2);
    let window_bmp = graphics::WindowBuffer::with_dimensions(window.width(), window.height());
    let mut input = input::Input::default();
    let game_data_ptr = game::startup(window_bmp.width, window_bmp.height);
    let mut dt: f32 = 0.0;
   
    let mut running = true;
    let mut went_inactive = false;
    while running {
        let frame_counter = time::Counter::start();

        running = window::dispatch_messages();
        if window.is_active() {
            went_inactive = false;
            input.update(&window);
        } else if !went_inactive {
            went_inactive = true;
            input.reset();
        } else {
            went_inactive = false;
        }

        game::update_and_render(game_data_ptr, &mut window, window_bmp, &input, dt);
        window.blit(window_bmp);

        dt = frame_counter.elapsed().as_secs() as f32;
    }
}
