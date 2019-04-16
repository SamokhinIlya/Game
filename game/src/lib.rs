extern crate core;
extern crate utils;
extern crate rusttype;

#[macro_use]
mod vector;
mod tilemap;
mod bitmap;
mod render;
mod file;

use platform::input::{Input, KBKey, MouseKey};
use utils::*;
use crate::{
    render::Color,
    vector::{
        prelude::*,
        distance_sq,
    },
    tilemap::{
        Tilemap,
        Tile,
        TileInfo,
        screen_pos_to_tilemap_pos,
        tilemap_pos_to_screen_pos,
    },
    bitmap::Bitmap,
    file::{Load, Save},
};
use crate::render::text::FontBitmaps;

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
    pub state: GameState,

    pub tilemap: Tilemap,
    pub tile_info: TileInfo,

    pub camera_pos: V2f,

    pub player: Entity,

    pub player_attack: Entity,
    pub player_attack_counter: f32,
    pub player_attack_prev_pos: V2f,

    pub enemies: [Entity; 1],

    pub player_bmps: PlayerBmps,
    pub enemy_bmp_right: Bitmap,
    pub enemy_bmp_left: Bitmap,

    pub font_bmp: render::text::FontBitmaps,

    pub c: f32,
    pub col: Color,
}

struct PlayerBmps {
    pub right: Bitmap,
    pub left: Bitmap,
    pub attack_right: Bitmap,
    pub attack_left: Bitmap,
}

pub fn startup(_screen_width: i32, _screen_height: i32) -> *mut () {
    let result = Box::new(GameData {
        state: GameState::LevelEditor,
        tilemap: Tilemap::load("data/levels/map_00")
            .unwrap_or_else(|_| Tilemap::new(
                15,
                15,
                //SCREEN_WIDTH_IN_TILES.ceil() as i32,
                //SCREEN_HEIGHT_IN_TILES.ceil() as i32,
            )),
        tile_info: TileInfo {
            size: 64,
            screen_width: 0.0,
            screen_height: 0.0,
            screen_width_in_px: 0,
            screen_height_in_px: 0,
            bmps: [Bitmap::load("data/sprites/size_64/test_tile.bmp").unwrap(); 1],
        },
        camera_pos: v2!(0.0, 0.0),
        player: Entity::with_pos_health(v2!(2.5, 2.5), 1),

        player_attack: Entity::with_pos_health(v2!(0.0, 0.0), 0),
        player_attack_counter: 0.0,
        player_attack_prev_pos: v2!(0.0, 0.0),

        enemies: [Entity::with_pos_health(v2!(3.5, 1.5), 5); 1],
        player_bmps: PlayerBmps {
            right: Bitmap::load("data/sprites/size_64/test_player_right.bmp").unwrap(),
            left: Bitmap::load("data/sprites/size_64/test_player_left.bmp").unwrap(),
            attack_right: Bitmap::load("data/sprites/size_64/test_player_attack_right.bmp").unwrap(),
            attack_left: Bitmap::load("data/sprites/size_64/test_player_attack_left.bmp").unwrap(),
        },
        enemy_bmp_right: Bitmap::load("data/sprites/size_64/test_enemy_right.bmp").unwrap(),
        enemy_bmp_left: Bitmap::load("data/sprites/size_64/test_enemy_left.bmp").unwrap(),
        font_bmp: render::text::FontBitmaps::new("data/fonts/Inconsolata-Regular.ttf", 20).unwrap(),
        c: 0.0,
        col: Color::WHITE,
    });

    Box::into_raw(result) as *mut ()
}

pub fn update_and_render(
    window_buffer: platform::graphics::WindowBuffer,
    input:         &Input,
    game_data:     *mut (),
) -> String {
    let mut window_bmp = Bitmap::from(window_buffer);
    #[allow(clippy::cast_ptr_alignment)]
    let data = unsafe { &mut *(game_data as *mut GameData) };
    let dt = input.dt;

    if data.tile_info.screen_width_in_px != window_bmp.width() {
        data.tile_info.screen_width_in_px = window_bmp.width();
        data.tile_info.screen_width =
            data.tile_info.screen_width_in_px as f32 / data.tile_info.size as f32;
    }
    if data.tile_info.screen_height_in_px != window_bmp.height() {
        data.tile_info.screen_height_in_px = window_bmp.height();
        data.tile_info.screen_height =
            data.tile_info.screen_height_in_px as f32 / data.tile_info.size as f32;
    }

    let info = match data.state {
        GameState::Playing => playing(&mut window_bmp, input, data, dt),
        GameState::LevelEditor => level_editor(&mut window_bmp, input, data, dt),
    };

    std::mem::forget(window_bmp);

    info
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

    // attack update ///////////////////////////////////////////////////////////////
    if data.player_attack.health.hp > 0 {
        for enemy in data.enemies.iter_mut() {
            if enemy.health.hp <= 0 {
                continue
            }
            if !aabb_collision(data.player_attack.rect(), enemy.rect()) {
                continue
            }

            match enemy.health.knockback {
                Knockback::No => {
                    enemy.health.hp -= 1;
                    enemy.health.knockback = Knockback::Knocked {
                        time_remaining: 1.0,
                        just_hit: true,
                    };
                },
                Knockback::Knocked {..} => (),
            }
        }

        data.player_attack_counter -= dt;
        if data.player_attack_counter < 0.0 {
            data.player_attack.health.hp = 0;
        }
    } else if input.keyboard[J].pressed() {
        data.player_attack_counter = 0.5;
        data.player_attack.health.hp = 1;
        data.player_attack.facing_direction = data.player.facing_direction;
        data.player_attack.pos = v2!(
            data.player.pos.x + match data.player.facing_direction {
                Direction::Right => 0.5,
                Direction::Left => -0.5,
            },
            data.player.pos.y,
        );
        data.player_attack_prev_pos = data.player_attack.pos;
    }

    // player movement //////////////////////////////////////////////////////////
    let player_command = {
        let (left, right) = (
            input.keyboard[A].is_down(),
            input.keyboard[D].is_down(),
        );
        let dir = match (left, right) {
            (false, true) => Some(Direction::Right),
            (true, false) => Some(Direction::Left),
            _             => None,
        };
        let jump = input.keyboard[K].pressed();
        MovementCommand::Platformer { dir, jump }
    };
    if let MovementCommand::Platformer { dir: Some(dir), .. } = player_command {
        data.player.facing_direction = dir;
    }
    data.player.mov(&data.tilemap, player_command, dt);
    
    /////////////////////////////////////////////////////////////////////////////
    /* attack movement */ {
        let dx = match data.player_attack.facing_direction {
            Direction::Right => 0.5,
            Direction::Left => -0.5,
        };
        //TODO: kill projectile when out of sight
        if h_tilemap_collision(&data.player_attack, &data.tilemap, dx).is_some() {
            data.player_attack.health.hp = 0;
        } else {
            data.player_attack.pos.x += dx;
        }
    }

    // enemy movement //////////////////////////////////////////////////////
    for enemy in data.enemies.iter_mut().filter(|x| x.health.hp > 0) {
        let enemy_command = match enemy.health.knockback {
            Knockback::No  => {
                if distance_sq(enemy.pos, data.player.pos) >= 16.0 {
                    let dir = if enemy.pos.x < data.player.pos.x {
                        Some(Direction::Right)
                    } else if enemy.pos.x > data.player.pos.x {
                        Some(Direction::Left)
                    } else {
                        None
                    };
                    let jump = enemy.pos.y < data.player.pos.y;

                    MovementCommand::Platformer { dir, jump }
                } else {
                    MovementCommand::Platformer { dir: None, jump: false, }
                }
            },
            Knockback::Knocked { time_remaining, just_hit: true } => {
                enemy.health.knockback = Knockback::Knocked {
                    time_remaining,
                    just_hit: false,
                };

                let force = 100.0;
                MovementCommand::Velocity(v2!(
                    match enemy.facing_direction {
                        Direction::Left => force,
                        Direction::Right => -force,
                    },
                    force * 3.0,
                ))
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

                MovementCommand::Acceleration(v2!(0.0, 0.0))
            },
        };
        if let MovementCommand::Platformer { dir: Some(dir), .. } = enemy_command {
            enemy.facing_direction = dir;
        }
        enemy.mov(&data.tilemap, enemy_command, input.dt);
    }

    ///////////////////////////////////////////////////////////////
    /* camera movement */ {
        let screen_center = v2!(data.tile_info.screen_width, data.tile_info.screen_height) * 0.5;

        // camera origin is bottom left corner of a screen
        data.camera_pos = data.player.pos - screen_center;
        clamp(
            &mut data.camera_pos.x,
            0.0,
            data.tilemap.width() as f32 - data.tile_info.screen_width,
        );
        clamp(
            &mut data.camera_pos.y,
            0.0,
            data.tilemap.height() as f32 - data.tile_info.screen_height,
        );
    }

    // draw ////////////////////////////////////////////////////////
    render::clear(screen, Color::BLACK);

    data.tilemap.draw(screen, data.camera_pos, &data.tile_info);

    let bmp = match data.player.facing_direction {
        Direction::Right => &data.player_bmps.right,
        Direction::Left => &data.player_bmps.left,
    };
    data.player.draw(screen, bmp, data.camera_pos, data.tile_info.size);

    let rect = data.player.rect();
    let min = tilemap_pos_to_screen_pos(rect.min, data.camera_pos, screen.dim(), data.tile_info.size);
    let max = tilemap_pos_to_screen_pos(rect.max, data.camera_pos, screen.dim(), data.tile_info.size);
    data.c += dt;
    if data.c > 0.2 {
        data.c = 0.0;
        data.col = match data.col {
            Color::WHITE => Color::BLACK,
            Color::BLACK => Color::WHITE,
            _ => unreachable!(),
        };
    }
    render::draw_rect(screen, v2!(min.x, max.y), v2!(max.x, min.y), data.col, 1);

    if data.player_attack.health.hp > 0 {
        let bmp = match data.player_attack.facing_direction {
            Direction::Right => &data.player_bmps.attack_right,
            Direction::Left => &data.player_bmps.attack_left,
        };
        data.player_attack.draw(screen, bmp, data.camera_pos, data.tile_info.size);
    }

    for enemy in &data.enemies {
        let bmp = match enemy.facing_direction {
            Direction::Right => &data.enemy_bmp_right,
            Direction::Left => &data.enemy_bmp_left,
        };
        match enemy.health.knockback {
            Knockback::Knocked { time_remaining, .. }
                if (time_remaining * 20.0).sin() > 0.0 => (),
            _ => enemy.draw(screen, bmp, data.camera_pos, data.tile_info.size),
        }
        let rect = enemy.rect();
        let min = tilemap_pos_to_screen_pos(rect.min, data.camera_pos, screen.dim(), data.tile_info.size);
        let max = tilemap_pos_to_screen_pos(rect.max, data.camera_pos, screen.dim(), data.tile_info.size);
        render::draw_rect(screen, v2!(min.x, max.y), v2!(max.x, min.y), Color::WHITE, 1);
    }

    format!(" {}", data.enemies[0])
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
        if save_result.is_err() {
            //TODO: error info
            data.font_bmp.draw_string(screen, v2!(10, 10), "Error saving bitmap");
        }
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
            (false, true ) => data.camera_pos.x += CAMERA_SPEED * dt,
            (true , false) => data.camera_pos.x -= CAMERA_SPEED * dt,
            _ => (),
        }
        match (input.keyboard[KBKey::S].is_down(), input.keyboard[KBKey::W].is_down()) {
            (false, true ) => data.camera_pos.y += CAMERA_SPEED * dt,
            (true , false) => data.camera_pos.y -= CAMERA_SPEED * dt,
            _ => (),
        }
    }

    let mouse: V2i = screen_pos_to_tilemap_pos(
        input.mouse.pos().into(),
        data.camera_pos,
        screen.dim(),
        data.tile_info.size,
    ).trunc().into();

    let mouse_pos_textbox = if input.mouse.pos().0 >= 0 && input.mouse.pos().0 < screen.width()
        && input.mouse.pos().1 >= 0 && input.mouse.pos().1 < screen.height()
    {
        let pos: V2i = input.mouse.pos().into();
        let margin = v2!(10);
        let mut text_pos = pos + margin;

        let text = format!("{} : {}", mouse.x, mouse.y);
        let width = data.font_bmp.width(&text);
        let height = data.font_bmp.height();

        // move textbox, so that it doesn't intersect edges of a screen
        if text_pos.x + width > screen.width() {
            text_pos.x = pos.x - width - margin.x;
        }
        if text_pos.y + height > screen.height() {
            text_pos.y = pos.y - height - margin.y;
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
        if mouse.x >= 0 && mouse.x < data.tilemap.width()
            && mouse.y >= 0 && mouse.y < data.tilemap.height()
        {
            data.tilemap[(mouse.x, mouse.y)] = tile;
        }
    }

    render::clear(screen, Color::BLACK);

    data.tilemap.draw(screen, data.camera_pos, &data.tile_info);
    data.tilemap.draw_grid(screen, data.camera_pos, &data.tile_info);
    data.tilemap.draw_outline(screen, data.camera_pos, &data.tile_info);

    fn draw_text_box(
        dst: &mut Bitmap,
        font: &FontBitmaps,
        text: &str,
        p: V2i
    ) -> V2i {
        const MARGIN: V2i = v2!(5, 5);

        //TODO: get_bbox method?
        let min_text_box = p;
        let max_text_box = min_text_box
            + v2!(font.width(text), font.height())
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
        v2!(50),
    );

    let _ = draw_text_box(
        screen,
        &data.font_bmp,
        "Use arrow keys to change tilemap size.",
        v2!(50, bottom_left.y),
    );

    if let Some((text, pos)) = mouse_pos_textbox {
        draw_text_box(screen, &data.font_bmp, &text, pos);
    }

    // draw yellow outline
    render::draw_rect(screen, v2!(0), screen.dim(), Color::YELLOW, 5);

    format!(" camera: {:?}", data.camera_pos)
}

#[derive(Copy, Clone, Debug)]
struct Entity {
    pub pos: V2f,
    pub vel: V2f,

    pub bottom_left_offset: V2f,
    pub size: V2f,

    pub facing_direction: Direction,
    pub health: Health,
    pub movement_state: MovementState,
}

#[derive(Copy, Clone, Debug)]
enum Direction {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
enum MovementState {
    OnTheGround,
    InTheAir { jumped_again: bool },
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

impl Entity {
    const MAX_VELOCITY: V2f = v2!(6.0, 13.0);

    pub fn new() -> Self {
        Self {
            pos: v2!(1.5, 1.5),
            vel: v2!(0.0, 0.0),

            // TODO: something about this hardcoding
            bottom_left_offset: v2!(-(0.75 * 0.5), -(0.5 - 0.001)),
            size: v2!(0.75, (0.5 - 0.001) + (0.5 - 1.0 / 9.0)),

            health: Health { hp: 1, knockback: Knockback::No },
            facing_direction: Direction::Right,
            movement_state: MovementState::InTheAir { jumped_again: true },
        }
    }

    pub fn rect(&self) -> Rect2 {
        let bottom_left = self.pos + self.bottom_left_offset;
        Rect2::from_bbox(
            bottom_left,
            bottom_left + self.size,
        )
    }

    pub fn with_pos_health(pos: V2f, hp: i32) -> Self {
        let mut entity = Self::new();
        entity.pos = pos;
        entity.health.hp = hp;
        entity
    }

    pub fn draw(&self, screen: &Bitmap, bmp: &Bitmap, camera: V2f, tile_size: i32) {
        let screen_pos = tilemap_pos_to_screen_pos(self.pos, camera, screen.dim(), tile_size);
        render::draw_bmp(screen, bmp, screen_pos - v2!(tile_size / 2));
    }

    //TODO: derive consts from height and length of a desired jump
    pub fn mov(&mut self, tilemap: &Tilemap, command: MovementCommand, dt: f32) {
        use std::ops::{Add, Mul};

        const HORIZONTAL_ACC: f32 = 50.0;
        const JUMP_VEL: f32 = 40.0;
        const GRAVITY: V2f = v2!(0.0, -50.0);

        fn friction(vel: f32) -> f32 {
            vel * -8.0
        }

        fn delta_position<T>(acc: T, vel: T, dt: f32) -> T
            where T: Mul<f32, Output=T> + Add<Output=T>,
                  f32: Mul<T, Output=T>,
        {
            0.5 * acc * dt*dt + vel * dt
        }

        fn delta_velocity<T>(acc: T, dt: f32) -> T
            where T: Mul<f32, Output=T>
        {
            acc * dt
        }

        let (V2 { x: mut dx, y: mut dy }, mut new_vel) = match command {
            // TODO: do different things based on MovementState
            MovementCommand::Acceleration(mut a) => {
                a += v2!(friction(self.vel.x), 0.0);
                (delta_position(a, self.vel, dt), self.vel + delta_velocity(a, dt))
            },
            // TODO: do different things based on MovementState
            MovementCommand::Velocity(new_vel) => {
                (delta_position(v2!(0.0, 0.0), new_vel, dt), new_vel)
            },
            MovementCommand::Platformer { dir, jump } => {
                let base_acc_x = HORIZONTAL_ACC * match dir {
                    Some(Direction::Right) => 1.0,
                    Some(Direction::Left) => -1.0,
                    None                   => 0.0,
                };
                match self.movement_state {
                    MovementState::OnTheGround => {
                        let acc_x = base_acc_x + friction(self.vel.x);
                        let new_velx = self.vel.x + delta_velocity(acc_x, dt);
                        if jump {
                            self.movement_state = MovementState::InTheAir {
                                jumped_again: false,
                            };

                            (
                                delta_position(v2!(acc_x, 0.0), self.vel, dt),
                                v2!(new_velx, JUMP_VEL),
                            )
                        } else {
                            (
                                // -0.01 is for checking if there is a tile underneath
                                v2!(delta_position(acc_x, self.vel.x, dt), -0.01),
                                v2!(new_velx, 0.0),
                            )
                        }
                    },
                    MovementState::InTheAir { jumped_again: false } if jump => {
                        self.movement_state = MovementState::InTheAir {
                            jumped_again: true,
                        };
                        let acc_x = base_acc_x;
                        let new_vel = v2!(
                            self.vel.x + delta_velocity(acc_x, dt),
                            0.75 * JUMP_VEL,
                        );
                        let acc = v2!(acc_x, 0.0);

                        (delta_position(acc, self.vel, dt), new_vel)
                    },
                    MovementState::InTheAir { .. } => {
                        let acc = {
                            let acc_x = {
                                let air_movement_penalty = base_acc_x * 0.8;
                                base_acc_x - air_movement_penalty
                            };
                            v2!(acc_x, GRAVITY.y)
                        };

                        (delta_position(acc, self.vel, dt), self.vel + delta_velocity(acc, dt))
                    },
                }
            },
        };

        if dx != 0.0 {
            if let Some(tile_x) = h_tilemap_collision(self, tilemap, dx).map(|x| x as f32) {
                new_vel.x = 0.0;
                dx = if dx > 0.0 {
                    tile_x - self.pos.x - (self.size.x + self.bottom_left_offset.x) * 1.01
                } else {
                    tile_x + 1.0 - self.pos.x - self.bottom_left_offset.x * 1.01
                }
            }
        }
        self.pos.x += dx;

        if dy != 0.0 {
            if let Some(tile_y) = v_tilemap_collision(self, tilemap, dy).map(|y| y as f32) {
                new_vel.y = 0.0;
                dy = if dy > 0.0 {
                    tile_y - self.pos.y - (self.size.y + self.bottom_left_offset.y) * 1.01
                } else {
                    self.movement_state = MovementState::OnTheGround;
                    tile_y + 1.0 - self.pos.y - self.bottom_left_offset.y * 1.01
                }
            } else if let MovementState::OnTheGround = self.movement_state {
                self.movement_state = MovementState::InTheAir { jumped_again: false };
            }
        }
        self.pos.y += dy;

        self.vel = new_vel;
        clamp(&mut self.vel.x, -Entity::MAX_VELOCITY.x, Entity::MAX_VELOCITY.x);
        clamp(&mut self.vel.y, -Entity::MAX_VELOCITY.y, Entity::MAX_VELOCITY.y);
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let airborne = match self.movement_state {
            MovementState::OnTheGround                      => "_",
            MovementState::InTheAir { jumped_again: false } => "-",
            MovementState::InTheAir { jumped_again: true }  => "^",
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
    Acceleration(V2f),
    Velocity(V2f),
    Platformer { dir: Option<Direction>, jump: bool },
}

#[derive(Copy, Clone, Debug)]
struct Rect2 {
    pub min: V2f,
    pub max: V2f,
}

impl Rect2 {
    pub fn right(self)  -> f32 { self.max.x }
    pub fn left(self)   -> f32 { self.min.x }
    pub fn top(self)    -> f32 { self.max.y }
    pub fn bottom(self) -> f32 { self.min.y }
    
    pub fn from_bbox(bottom_left: V2f, top_right: V2f) -> Self {
        Self {
            min: bottom_left,
            max: top_right,
        }
    }
}

fn aabb_collision(rect0: Rect2, rect1: Rect2) -> bool {
    rect0.right() > rect1.left()
        && rect0.left() < rect1.right()
        && rect0.top() > rect1.bottom()
        && rect0.bottom() < rect1.top()
}

fn h_tilemap_collision(entity: &Entity, tilemap: &Tilemap, dx: f32) -> Option<i32> {
    assert_ne!(dx, 0.0, "Collision dx");

    let u_tile_y = (entity.pos.y + entity.size.y + entity.bottom_left_offset.y).floor() as i32;
    let d_tile_y = (entity.pos.y + entity.bottom_left_offset.y           ).floor() as i32;

    let (offset, step_x) = if dx > 0.0 {
        (entity.size.x + entity.bottom_left_offset.x, 1)
    } else {
        (entity.bottom_left_offset.x, -1)
    };
    let from_x = (entity.pos.x + offset).floor() as i32;
    let to_x = (entity.pos.x + offset + dx).floor() as i32;

    let mut tile_x = from_x;
    loop {
        match (
            tilemap.get(tile_x, u_tile_y),
            tilemap.get(tile_x, d_tile_y),
        ) {
            (Some(up), Some(dn)) if up.is_obstacle() || dn.is_obstacle() =>
                return Some(tile_x),
            (None, _   ) |
            (_   , None) =>
                return Some(tile_x),
            _ => (),
        }

        if tile_x == to_x {
            break
        }
        tile_x += step_x;
    }

    None
}

fn v_tilemap_collision(entity: &Entity, tilemap: &Tilemap, dy: f32) -> Option<i32> {
    assert_ne!(dy, 0.0, "Collision dy");

    let r_tile_x = (entity.pos.x + entity.size.x + entity.bottom_left_offset.x).floor() as i32;
    let l_tile_x = (entity.pos.x + entity.bottom_left_offset.x).floor() as i32;

    let (offset, step_y) = if dy > 0.0 {
        (entity.size.y + entity.bottom_left_offset.y, 1)
    } else {
        (entity.bottom_left_offset.y, -1)
    };
    let from_y = (entity.pos.y + offset).floor() as i32;
    let to_y = (entity.pos.y + offset + dy).floor() as i32;

    let mut tile_y = from_y;
    loop {
        match (
            tilemap.get(r_tile_x, tile_y),
            tilemap.get(l_tile_x, tile_y),
        ) {
            (Some(rt), Some(lt)) if rt.is_obstacle() || lt.is_obstacle() =>
                return Some(tile_y),
            (None, _   ) |
            (_   , None) =>
                return Some(tile_y),
            _ => (),
        }

        if tile_y == to_y {
            break
        }
        tile_y += step_y;
    }

    None
}
