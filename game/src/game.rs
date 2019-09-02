mod tilemap;

use platform::input::{Input, KBKey, MouseKey};
use utils::*;
use crate::{
    render::{
        self,
        Color,
        Bitmap,
        text::FontBitmaps,
        screen_info::ScreenInfo,
    },
    geom::{
        vector::{
            prelude::*,
            distance_sq,
        },
        matrix::Mat2,
        aabb::AABB,
    },
    file::{Load, Save},
};
use tilemap::{
    Tilemap,
    Tile,
    TileInfo,
};

/* TODO: in progress
  - level editor with screen_scale != 1
*/

/* TODO: next
    - game:
        - finish attacking

    - engine:
        - timer
        - interface (menus, buttons, etc)
        - rendering api:
            - abstract away coordinate system handling
            - command buffer
            - something about bitmaps
            - text?

        - transformations:
            - scaling
            - rotation
        - fps lock?
*/

/* TODO: ideas
  - level editor
  - move to ecs
  - visuals:
    - dust cloud when changing direction
    - generating vfx at runtime (particles)
*/

enum GameState {
    Playing,
    LevelEditor,
}

struct GameData {
    pub screen_info: ScreenInfo,

    pub state: GameState,

    pub tilemap: Tilemap,
    pub tile_info: TileInfo,

    pub player: Entity,
    pub hook: Entity,

    pub player_attack_counter: f32,

    pub enemies: [Entity; 1],

    pub player_bmps: PlayerBmps,
    pub enemy_bmp_right: Bitmap,
    pub enemy_bmp_left: Bitmap,

    pub font_bmp: render::text::FontBitmaps,
    pub text: String,
    pub text_timer: f32,
}

struct PlayerBmps {
    pub right: Bitmap,
    pub left: Bitmap,
    pub hook: Bitmap,
}

fn restart(data: &mut GameData) {
    data.player = Entity::new_character((2.5, 2.5).into(), 1);
}

pub fn startup(_screen_width: i32, _screen_height: i32) -> *mut () {
    const SPRITE_FOLDER: &str = "data/sprites/size_16/";
    let tile_size = 16;
    let hook_bmp = Bitmap::load(format!("{}{}", SPRITE_FOLDER, "hook.png")).unwrap();

    let mut result = Box::new(GameData {
        screen_info: ScreenInfo {
            width: i32::default(),
            height: i32::default(),
            scale: 4,
            game_to_screen_matrix: Mat2::<f32>::from([
                [tile_size as f32,  0.0             ],
                [0.0             , -tile_size as f32],
            ]),
            screen_to_game_matrix: Mat2::<f32>::from([
                [(tile_size as f32).recip(),  0.0                       ],
                [0.0                       , (-tile_size as f32).recip()],
            ]),
            camera: (0.0, 0.0).into(),
        },

        state: GameState::LevelEditor,
        tilemap: Tilemap::load("data/levels/map_00").unwrap_or_else(|_| Tilemap::new(15, 15)),
        tile_info: TileInfo {
            size: tile_size,
            screen_width: 0.0,
            screen_height: 0.0,
            bmps: [Bitmap::load(format!("{}{}", SPRITE_FOLDER, "test_ground.png")).unwrap(); 1],
        },

        player: Entity::new_character((2.5, 2.5).into(), 1),
        hook: {
            let pixel_size = 1.0 / tile_size as f32;

            let width = pixel_size * hook_bmp.width() as f32;
            let height = pixel_size * hook_bmp.height() as f32;

            Entity::new_thing(
                (2.5, 2.5).into(),
                (width / 2.0, height / 2.0).into(),
                (width, height).into(),
            )
        },

        player_attack_counter: 0.0,

        enemies: [Entity::new_character((3.5, 1.5).into(), 5); 1],
        player_bmps: PlayerBmps {
            right: Bitmap::load(format!("{}{}", SPRITE_FOLDER, "test_player_right.png")).unwrap(),
            left: Bitmap::load(format!("{}{}", SPRITE_FOLDER, "test_player_left.png")).unwrap(),
            hook: hook_bmp,
        },
        enemy_bmp_right: Bitmap::load(format!("{}{}", SPRITE_FOLDER, "test_enemy_right.png")).unwrap(),
        enemy_bmp_left: Bitmap::load(format!("{}{}", SPRITE_FOLDER, "test_enemy_left.png")).unwrap(),
        font_bmp: render::text::FontBitmaps::new("data/fonts/Inconsolata-Regular.ttf", 20).unwrap(),
        text: String::new(),
        text_timer: 0.0,
    });
    restart(result.as_mut());

    // FIXME: ugh
    Box::into_raw(result) as *mut ()
}

pub fn update_and_render(
    game_data:     *mut (),
    window:        &mut platform::window::Window,
    window_buffer: platform::graphics::WindowBuffer,
    input:         &Input,
    dt:            f32,
) {
    #[allow(clippy::cast_ptr_alignment)]
    let data = unsafe {
        &mut *(game_data as *mut GameData)
    };

    if (input.keyboard[KBKey::Alt].is_down() && input.keyboard[KBKey::Enter].pressed())
        || input.keyboard[KBKey::F11].pressed()
    {
        window.toggle_fullscreen();
    }
    if input.keyboard[KBKey::F12].pressed() {
        data.screen_info.scale = if data.screen_info.scale == 1 { 4 } else { 1 };
    }

    let mut window_bmp = Bitmap::from(window_buffer);
    // FIXME: alloc dealloc every frame
    let mut draw_bmp = Bitmap::with_dimensions(window_bmp.width() / data.screen_info.scale, window_bmp.height() / data.screen_info.scale);

    if data.screen_info.width != draw_bmp.width() {
        data.screen_info.width = draw_bmp.width();
        data.tile_info.screen_width =
            data.screen_info.width as f32 / data.tile_info.size as f32;
    }
    if data.screen_info.height != draw_bmp.height() {
        data.screen_info.height = draw_bmp.height();
        data.tile_info.screen_height =
            data.screen_info.height as f32 / data.tile_info.size as f32;
    }

    let info = match data.state {
        GameState::Playing => playing(&mut draw_bmp, input, data, dt),
        GameState::LevelEditor => level_editor(&mut draw_bmp, input, data, dt),
    };

    render::scale_up(&draw_bmp, &mut window_bmp, data.screen_info.scale);

    window.set_title(unsafe {
        &std::ffi::CString::from_vec_unchecked(
            format!(
                "frame: {:>3.3} ms, {:>2.2} fps || {}\0",
                dt * 1000.0,
                dt.recip(),
                info,
            ).into()
        )
    });

    std::mem::forget(window_bmp);
}

#[allow(clippy::useless_format)]
fn playing(
    screen: &mut Bitmap,
    input:  &Input,
    data:   &mut GameData,
    dt:     f32,
) -> String {
    use KBKey::*;

    if input.keyboard[K].pressed() && input.keyboard[Ctrl].is_down() {
        data.state = GameState::LevelEditor;
        render::clear(screen, Color::BLACK);
    }
    if input.keyboard[Escape].pressed() {
        restart(data);
    }

    // attack update ///////////////////////////////////////////////////////////////
    // FIXME: -0.3 is cooldown time
    if data.player_attack_counter >= -0.3 {
        data.player_attack_counter -= dt;
    // FIXME: with this setup attack will damage enemy only on the next frame
    } else if input.keyboard[J].pressed() && data.player_attack_counter < -0.3 {
        data.player_attack_counter = 0.3;
    }

    let attack_aabb = if data.player_attack_counter > 0.0 {
        let attack_offset = match data.player.facing {
            Direction::Left => (-1.0, 0.0).into(),
            Direction::Right => (1.0, 0.0).into(),
        };
        let attack_aabb = data.player.collision_aabb().translate(attack_offset);
        data.enemies.iter_mut()
            .filter(|enemy| enemy.health.hp > 0 && aabb_collision(attack_aabb, enemy.collision_aabb()))
            .for_each(|enemy| match enemy.health.knockback {
                Knockback::Knocked { .. } => (),
                Knockback::No => {
                    enemy.health.hp -= 1;
                    enemy.health.knockback = Knockback::Knocked {
                        time_remaining: 1.0,
                        just_hit: true,
                    };
                },
            });
        Some((attack_aabb, attack_offset))
    } else {
        None
    };

    // player movement //////////////////////////////////////////////////////////
    let player_command = Some(MovementCommand::Platformer {
        dir: match (input.keyboard[A].is_down(), input.keyboard[D].is_down()) {
            (false, true) => Some(Direction::Right),
            (true, false) => Some(Direction::Left),
            _             => None,
        },
        jump: input.keyboard[K].pressed(),
    });
    if let Some(MovementCommand::Platformer { dir: Some(dir), .. }) = player_command {
        data.player.facing  = dir;
    }
    data.player.mov(&data.tilemap, player_command, dt);

    // enemy movement //////////////////////////////////////////////////////
    for enemy in data.enemies.iter_mut().filter(|x| x.health.hp > 0) {
        let enemy_command = Some(match enemy.health.knockback {
            Knockback::No if distance_sq(enemy.pos, data.player.pos) >= 16.0 => {
                MovementCommand::Platformer {
                    dir: if enemy.pos.x < data.player.pos.x {
                        Some(Direction::Right)
                    } else if enemy.pos.x > data.player.pos.x {
                        Some(Direction::Left)
                    } else {
                        None
                    },
                    jump: enemy.pos.y < data.player.pos.y,
                }
            }
            Knockback::No => {
                MovementCommand::Platformer { dir: None, jump: false }
            },
            Knockback::Knocked { time_remaining, just_hit: true } => {
                enemy.health.knockback = Knockback::Knocked {
                    time_remaining,
                    just_hit: false,
                };

                let force = 100.0;
                MovementCommand::Velocity((
                    match enemy.facing {
                        Direction::Left => force,
                        Direction::Right => -force,
                    },
                    force * 3.0,
                ).into())
            },
            Knockback::Knocked { mut time_remaining, just_hit: false } => {
                time_remaining -= dt;
                enemy.health.knockback = if time_remaining <= 0.0 {
                    Knockback::No
                } else {
                    Knockback::Knocked {
                        time_remaining,
                        just_hit: false,
                    }
                };

                MovementCommand::Platformer { dir: None, jump: false }
            },
        });
        if let Some(MovementCommand::Platformer { dir: Some(dir), .. }) = enemy_command {
            enemy.facing = dir;
        }
        enemy.mov(&data.tilemap, enemy_command, dt);
    }

    ///////////////////////////////////////////////////////////////
    /* camera movement */ {
        let screen_center = V2f::new(data.tile_info.screen_width, data.tile_info.screen_height) * 0.5;

        // camera origin is bottom left corner of a screen
        data.screen_info.camera = data.player.pos - screen_center;
        data.screen_info.camera.x = clamp(
            data.screen_info.camera.x,
            0.0,
            data.tilemap.width() as f32 - data.tile_info.screen_width,
        );
        data.screen_info.camera.y = clamp(
            data.screen_info.camera.y,
            0.0,
            data.tilemap.height() as f32 - data.tile_info.screen_height,
        );
    }

    // draw ////////////////////////////////////////////////////////
    render::clear(screen, Color::BLACK);

    data.tilemap.draw(screen, &data.screen_info, &data.tile_info);

    let player_bmp = match data.player.facing  {
        Direction::Right => &data.player_bmps.right,
        Direction::Left => &data.player_bmps.left,
    };

    let player_up_left = render::v2_to_screen(data.player.pos, &data.screen_info) - V2::diag(data.tile_info.size / 2);
    render::draw_bmp(screen, player_bmp, player_up_left);

    let player_collision_rect = render::aabb_to_screen(data.player.collision_aabb(), &data.screen_info);
    render::draw_rect(screen, player_collision_rect.min, player_collision_rect.max, Color::YELLOW, 1);

    // attack collision box
    if let Some((attack_aabb, _)) = attack_aabb {
        let AABB { min, max } = render::aabb_to_screen(attack_aabb, &data.screen_info);
        render::fill_rect(screen, min, max, { let mut c = Color::RED; c.a = 0x77; c });
    }

    if let Some((_, attack_offset)) = attack_aabb {
        let bmp = &data.player_bmps.hook;
        let attack_pos = data.player.collision_aabb().top_left() + attack_offset;
        let attack_screen_pos = render::v2_to_screen(attack_pos, &data.screen_info);
        render::draw_bmp(screen, bmp, attack_screen_pos);
    }

    for enemy in &data.enemies {
        match enemy.health.knockback {
            Knockback::Knocked { time_remaining, .. } if (time_remaining * 20.0).sin() > 0.0 => (),
            _ => {
                let pos = render::v2_to_screen(enemy.pos, &data.screen_info);
                let bmp = match enemy.facing {
                    Direction::Right => &data.enemy_bmp_right,
                    Direction::Left => &data.enemy_bmp_left,
                };
                render::draw_bmp(screen, bmp, pos - V2::diag(data.tile_info.size / 2));
            },
        }

        // enemy collision box
        let AABB { min, max } = render::aabb_to_screen(enemy.collision_aabb(), &data.screen_info);
        render::draw_rect(screen, min, max, Color::YELLOW, 1);
    }

    format!(" {}", data.player.pos.x + data.player.origin_to_bottom_left.x)
}

#[allow(clippy::useless_format)]
fn level_editor(
    screen: &mut Bitmap,
    input:  &Input,
    data:   &mut GameData,
    dt:     f32,
) -> String {
    if input.keyboard[KBKey::K].pressed() && input.keyboard[KBKey::Ctrl].is_down() {
        data.state = GameState::Playing;
    }

    if input.keyboard[KBKey::S].pressed() && input.keyboard[KBKey::Ctrl].is_down() {
        let save_result = data.tilemap.save("data/levels/map_00");
        data.text_timer = 1.0;
        data.text = if save_result.is_err() {
            //TODO: error info
            "Error saving bitmap".into()
        } else {
            "Saved".into()
        };
    }

    if data.text_timer > 0.0 {
        data.text_timer -= dt;
        //FIXME: doesnt work
        data.font_bmp.draw_string(screen, (10, 10).into(), &data.text);
    }

    let mut new_tilemap_size = data.tilemap.dim();
    match (input.keyboard[KBKey::Right].pressed(), input.keyboard[KBKey::Left].pressed()) {
        (true, false) => new_tilemap_size.x += 1,
        (false, true) => new_tilemap_size.x -= 1,
        _ => (),
    }
    match (input.keyboard[KBKey::Up].pressed(), input.keyboard[KBKey::Down].pressed()) {
        (true, false) => new_tilemap_size.y += 1,
        (false, true) => new_tilemap_size.y -= 1,
        _ => (),
    }
    if new_tilemap_size != data.tilemap.dim()
        && new_tilemap_size.x > 0
        && new_tilemap_size.y > 0
    {
        data.tilemap.resize(new_tilemap_size.x, new_tilemap_size.y);
    }

    if !input.keyboard[KBKey::Ctrl].is_down() {
        const CAMERA_SPEED: f32 = 10.0;
        match (input.keyboard[KBKey::A].is_down(), input.keyboard[KBKey::D].is_down()) {
            (false, true ) => data.screen_info.camera.x += CAMERA_SPEED * dt,
            (true , false) => data.screen_info.camera.x -= CAMERA_SPEED * dt,
            _ => (),
        }
        match (input.keyboard[KBKey::S].is_down(), input.keyboard[KBKey::W].is_down()) {
            (false, true ) => data.screen_info.camera.y += CAMERA_SPEED * dt,
            (true , false) => data.screen_info.camera.y -= CAMERA_SPEED * dt,
            _ => (),
        }
    }

    let mouse_screen = V2i::from(input.mouse.pos());
    let mouse_pos = V2f::from(mouse_screen) - V2f::new(0.0, data.screen_info.height as f32);
    let mouse = V2i::from(&data.screen_info.screen_to_game_matrix * mouse_pos + data.screen_info.camera);

    let mouse_pos_textbox: Option<(String, V2i)> = if (0..screen.width()).contains(&mouse_screen.x)
        && (0..screen.height()).contains(&mouse_screen.y)
    {
        let margin = (10, 10).into();
        let mut text_pos = mouse_screen + margin;

        let text = format!("{} : {}", mouse.x, mouse.y);
        let width = data.font_bmp.width(&text);
        let height = data.font_bmp.height();

        // move textbox, so that it doesn't intersect edges of a screen
        if text_pos.x + width > screen.width() {
            text_pos.x = mouse_screen.x - width - margin.x;
        }
        if text_pos.y + height > screen.height() {
            text_pos.y = mouse_screen.y - height - margin.y;
        }

        Some((text, text_pos))
    } else {
        None
    };

    let maybe_tile = if input.mouse[MouseKey::LB].is_down() {
        Some(Tile::Ground)
    } else if input.mouse[MouseKey::RB].is_down() {
        Some(Tile::Empty)
    } else {
        None
    };

    if let Some(tile) = maybe_tile {
        if (0..data.tilemap.width()).contains(&mouse.x)
            && (0..data.tilemap.height()).contains(&mouse.y)
        {
            data.tilemap[(mouse.x, mouse.y)] = tile;
        }
    }

    render::clear(screen, Color::BLACK);

    data.tilemap.draw(screen, &data.screen_info, &data.tile_info);
    data.tilemap.draw_grid(screen, &data.screen_info, &data.tile_info);
    //FIXME: horizontal line upper pixel is not drawn
    data.tilemap.draw_outline(screen, &data.screen_info);

    fn draw_text_box(
        dst: &mut Bitmap,
        font: &FontBitmaps,
        text: &str,
        p: V2i
    ) -> V2i {
        const MARGIN: V2i = V2i { x: 5, y: 5 };

        //TODO: get_bbox method?
        let min_text_box = p;
        let max_text_box = min_text_box
            + (font.width(text), font.height()).into()
            + MARGIN * 2;
        render::fill_rect(dst, min_text_box, max_text_box, Color::BLACK);
        render::draw_rect(dst, min_text_box, max_text_box, Color::WHITE, 1);
        font.draw_string(dst, min_text_box + MARGIN, text);

        max_text_box
    }

    let bottom_left = draw_text_box(
        screen,
        &data.font_bmp,
        &format!("{}x{}", data.tilemap.width(), data.tilemap.height()),
        (50, 50).into(),
    );

    let _ = draw_text_box(
        screen,
        &data.font_bmp,
        "Use arrow keys to change tilemap size.",
        (50, bottom_left.y).into(),
    );

    if let Some((text, pos)) = mouse_pos_textbox {
        draw_text_box(screen, &data.font_bmp, &text, pos);
    }

    // draw yellow outline
    render::draw_rect(screen, (0, 0).into(), screen.dim(), Color::YELLOW, 2);

    format!(" text: {}", data.text_timer)
}

#[derive(Copy, Clone, Debug)]
enum Direction { Left, Right }

#[derive(Copy, Clone, Debug)]
enum MovementState {
    Ground,
    Air { jumped_again: bool },
}

#[derive(Copy, Clone, Debug)]
struct Health {
    hp: i32,
    knockback: Knockback,
}

#[derive(Copy, Clone, Debug)]
enum Knockback {
    No,
    Knocked {
        time_remaining: f32,
        just_hit: bool,
    },
}

#[derive(Copy, Clone, Debug)]
struct Entity {
    pub pos: V2f,
    pub vel: V2f,

    pub origin_to_bottom_left: V2f,
    pub size: V2f,

    pub facing: Direction,
    pub health: Health,
    pub movement_state: MovementState,
}

impl Entity {
    pub fn new_entity(pos: V2f, origin_to_bottom_left: V2f, size: V2f, hp: i32) -> Self {
        Self {
            pos,
            vel: (0.0, 0.0).into(),

            origin_to_bottom_left,
            size,

            health: Health { hp, knockback: Knockback::No },
            facing: Direction::Right,
            movement_state: MovementState::Air { jumped_again: true },
        }
    }

    pub fn new_character(pos: V2f, health: i32) -> Self {
        // TODO: something about this hardcoding
        let origin_to_bottom_left = (-(0.75 * 0.5), -(0.5 - 0.001)).into();
        let size = (0.75, (0.5 - 0.001) + (0.5 - 1.0 / 9.0)).into();
        Self::new_entity(pos, origin_to_bottom_left, size, health)
    }

    pub fn new_thing(pos: V2f, origin_to_bottom_left: V2f, size: V2f) -> Self {
        Self::new_entity(pos, origin_to_bottom_left, size, 1)
    }

    pub fn collision_aabb(&self) -> AABB<f32> {
        let bottom_left = self.pos + self.origin_to_bottom_left;
        AABB {
            min: bottom_left,
            max: bottom_left + self.size,
        }
    }

    pub fn mov(&mut self, tilemap: &Tilemap, command: Option<MovementCommand>, dt: f32) {
        use MovementCommand::{Platformer, Velocity};
        use MovementState::{Ground, Air};
        use Direction::{Left, Right};

        const MAX_VEL_X: f32 = 10.0;
        const MAX_VEL_Y: f32 = 30.0;

        fn is_obstacle(tile0: Option<Tile>, tile1: Option<Tile>) -> bool {
            use Tile::Empty;
            if let (Some(Empty), Some(Empty)) = (tile0, tile1) {
                false
            } else {
                true
            }
        }

        let (mut new_vel_x, mut new_vel_y) = match command {
            Some(Platformer { dir, jump }) => {
                const ACC_X: f32 = 100.0;
                const GRAVITY_ACC: f32 = -100.0;
                const JUMP_VEL: f32 = 15.0;

                let new_vel_x = {
                    let mut acc_x = match dir {
                        Some(Left) => -ACC_X,
                        Some(Right) => ACC_X,
                        None => 0.0,
                    };
                    acc_x += match self.movement_state {
                        // friction
                        Ground => -self.vel.x * 12.0,
                        // air movement penalty
                        Air { .. } => -acc_x * 0.8,
                    };

                    self.vel.x + 0.5 * acc_x * dt
                };
                let new_vel_y = match self.movement_state {
                    Ground if jump => {
                        self.movement_state = Air { jumped_again: false };
                        JUMP_VEL
                    },
                    Air { jumped_again: false } if jump => {
                        self.movement_state = Air { jumped_again: true };
                        JUMP_VEL
                    },
                    Ground => 0.0,
                    _ => self.vel.y + 0.5 * GRAVITY_ACC * dt,
                };

                (new_vel_x, new_vel_y)
            },
            Some(Velocity(vel)) => {
                if vel.y > 0.0 {
                    self.movement_state = Air { jumped_again: false };
                }

                vel.into()
            },
            None => (0.0, 0.0),
        };

        // FIXME: collision detection works only for speeds less than a tile per frame

        let mut new_pos_x = self.pos.x + self.vel.x * dt;

        // x collision detection
        let dx = new_pos_x - self.pos.x;
        if dx != 0.0 {
            let top_y = self.pos.y + self.origin_to_bottom_left.y + self.size.y;
            let bottom_y = top_y - self.size.y;

            if dx > 0.0 {
                let new_right_tile_x = (new_pos_x + self.origin_to_bottom_left.x + self.size.x).floor() as i32;

                let top_right_tile = tilemap.get(new_right_tile_x, top_y.floor() as i32);
                let bottom_right_tile = tilemap.get(new_right_tile_x, bottom_y.floor() as i32);

                if is_obstacle(top_right_tile, bottom_right_tile) {
                    new_vel_x = 0.0;
                    new_pos_x = new_right_tile_x as f32 - (self.origin_to_bottom_left.x + self.size.x) - 0.01;
                }
            } else if dx < 0.0 {
                let new_left_tile_x = (new_pos_x + self.origin_to_bottom_left.x).floor() as i32;

                let top_left_tile = tilemap.get(new_left_tile_x, top_y.floor() as i32);
                let bottom_left_tile = tilemap.get(new_left_tile_x, bottom_y.floor() as i32);

                if is_obstacle(top_left_tile, bottom_left_tile) {
                    new_vel_x = 0.0;
                    //TODO: check if 0.01 is needed for left and bottom collisions
                    new_pos_x = (new_left_tile_x + 1) as f32 - self.origin_to_bottom_left.x + 0.01;
                }
            }
        }

        // x update
        self.vel.x = clamp(new_vel_x, -MAX_VEL_X, MAX_VEL_X);
        self.pos.x = new_pos_x;


        let mut new_pos_y = self.pos.y + self.vel.y * dt;

        // y collision detection
        let dy = new_pos_y - self.pos.y;
        if dy != 0.0 {
            let left_x = self.pos.x + self.origin_to_bottom_left.x;
            let right_x = left_x + self.size.x;

            if dy > 0.0 {
                let new_top_tile_y = (new_pos_y + self.origin_to_bottom_left.y + self.size.y).floor() as i32;

                let top_left_tile = tilemap.get(left_x.floor() as i32, new_top_tile_y);
                let top_right_tile = tilemap.get(right_x.floor() as i32, new_top_tile_y);

                if is_obstacle(top_left_tile, top_right_tile) {
                    new_vel_y = 0.0;
                    new_pos_y = new_top_tile_y as f32 - (self.origin_to_bottom_left.y + self.size.y) - 0.01;
                }
            } else if dy < 0.0 {
                let new_bottom_tile_y = (new_pos_y + self.origin_to_bottom_left.y).floor() as i32;

                let bottom_left_tile = tilemap.get(left_x.floor() as i32, new_bottom_tile_y);
                let bottom_right_tile = tilemap.get(right_x.floor() as i32, new_bottom_tile_y);

                if is_obstacle(bottom_left_tile, bottom_right_tile) {
                    new_vel_y = 0.0;
                    new_pos_y = (new_bottom_tile_y + 1) as f32 - self.origin_to_bottom_left.y + 0.01;

                    // hit the floor -> now on the ground
                    self.movement_state = Ground;
                }
            }
        }

        // y update
        self.vel.y = clamp(new_vel_y, -MAX_VEL_Y, MAX_VEL_Y);
        self.pos.y = new_pos_y;

        // ground check //////////
        if let Ground = self.movement_state {
            let tile_under = tilemap.get(self.pos.x.floor() as i32, self.pos.y.floor() as i32 - 1);
            if let Some(Tile::Empty) = tile_under {
                self.movement_state = Air { jumped_again: false };
            }
        }
        // ground check //////////
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use MovementState::{Ground, Air};
        let airborne = match self.movement_state {
            Ground                      => "_",
            Air { jumped_again: false } => "-",
            Air { jumped_again: true }  => "^",
        };
        write!(
            f,
            "pos: ({:>+5.2}, {:>+5.2}), vel: ({:>+5.2}, {:>+5.2}) |{}|",
            self.pos.x, self.pos.y, self.vel.x, self.vel.y, airborne,
        )
    }
}

#[derive(Copy, Clone, Debug)]
enum MovementCommand {
    Velocity(V2f),
    Platformer { dir: Option<Direction>, jump: bool },
}

fn aabb_collision<T: Num32>(rect0: AABB<T>, rect1: AABB<T>) -> bool {
    rect0.right() > rect1.left()
        && rect0.left() < rect1.right()
        && rect0.top() > rect1.bottom()
        && rect0.bottom() < rect1.top()
}
