extern crate core;
extern crate utils;
extern crate rusttype;

#[macro_use]
mod vector;
mod tilemap;
mod bitmap;
mod render;
mod file;

use platform::{
    input::{Input, KBKey, MouseKey},
    RawPtr,
};
use utils::*;
use crate::{
    render::Color,
    vector::{
        V2,
        distance_sq,
    },
    tilemap::{
        Tilemap,
        Tile,
        TILE_SIZE,
        SCREEN_WIDTH_IN_TILES,
        SCREEN_HEIGHT_IN_TILES,
        screen_pos_to_tilemap_pos,
        tilemap_pos_to_screen_pos,
    },
    bitmap::Bitmap,
    file::{Load, write_to_file},
};

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
    pub camera_pos: V2,

    pub player: Entity,

    pub player_attack: Entity,
    pub player_attack_counter: f32,
    pub player_attack_prev_pos: V2,

    pub enemies: [Entity; 1],

    pub tile_bitmaps: [Bitmap; 1],
    pub player_bmps: PlayerBmps,
    pub enemy_bmp_right: Bitmap,
    pub enemy_bmp_left: Bitmap,

    pub font_bmp: render::FontBitmaps,
}

struct PlayerBmps {
    pub right: Bitmap,
    pub left: Bitmap,
    pub attack_right: Bitmap,
    pub attack_left: Bitmap,
}

pub fn startup(_screen_width: i32, _screen_height: i32) -> RawPtr {
    let result = Box::new(GameData {
        state: GameState::LevelEditor,
        tilemap: Tilemap::load("data/levels/map_00")
            .unwrap_or_else(|_| Tilemap::new()),
        camera_pos: v2!(0.0, 0.0),
        player: Entity::with_pos_health(v2!(2.5, 2.5), 1),

        player_attack: Entity::with_pos_health(V2::ZERO, 0),
        player_attack_counter: 0.0,
        player_attack_prev_pos: V2::ZERO,

        enemies: [Entity::with_pos_health(v2!(3.5, 1.5), 5); 1],
        tile_bitmaps: [Bitmap::load("data/sprites/size_64/test_tile.bmp").unwrap(); 1],
        player_bmps: PlayerBmps {
            right: Bitmap::load("data/sprites/size_64/test_player_right.bmp").unwrap(),
            left: Bitmap::load("data/sprites/size_64/test_player_left.bmp").unwrap(),
            attack_right: Bitmap::load("data/sprites/size_64/test_player_attack_right.bmp").unwrap(),
            attack_left: Bitmap::load("data/sprites/size_64/test_player_attack_left.bmp").unwrap(),
        },
        enemy_bmp_right: Bitmap::load("data/sprites/size_64/test_enemy_right.bmp").unwrap(),
        enemy_bmp_left: Bitmap::load("data/sprites/size_64/test_enemy_left.bmp").unwrap(),
        font_bmp: render::FontBitmaps::load("data/fonts/Inconsolata-Regular.ttf").unwrap(),
    });

    Box::into_raw(result) as RawPtr
}

pub fn update_and_render(
    window_buffer: platform::graphics::WindowBuffer,
    input:         &Input,
    game_data:     RawPtr,
) -> String {
    let mut window_bmp = Bitmap::from(window_buffer);
    #[allow(clippy::cast_ptr_alignment)]
    let data = unsafe { &mut *(game_data as *mut GameData) };
    let dt = input.dt;

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
        for enemy in data.enemies.iter_mut().filter(|x| x.health.hp > 0) {
            let size = data.player_attack.size;
            let pos = data.player_attack.pos;
            let player_attack_hitbox = Rect2::from_bbox(
                pos + v2!(size.left_offset, size.bottom_offset),
                pos + v2!(1.0, 0.0) + v2!(size.right_offset, size.top_offset),
            );
            let enemy_hurtbox = Rect2::from_center_size(enemy.pos, enemy.size);

            if aabb_collision(player_attack_hitbox, enemy_hurtbox) {
                match enemy.health.knockback {
                    Knockback::No => {
                        enemy.health.hp -= 1;
                        enemy.health.knockback = Knockback::Knocked{
                            time_remaining: 1.0,
                            just_hit: true,
                        };
                    },
                    Knockback::Knocked {..} => (),
                }
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
        if let Some(_) = h_tilemap_collision(&data.player_attack, &data.tilemap, dx) {
            data.player_attack.health.hp = 0;
        } else {
            data.player_attack.pos.x += dx;
        }
    }

    // enemy movement //////////////////////////////////////////////////////
    for enemy in data.enemies.iter_mut().filter(|x| x.health.hp > 0) {
        let enemy_command = match enemy.health.knockback {
            Knockback::No if distance_sq(enemy.pos, data.player.pos) >= 16.0 => {
                let dir = if enemy.pos.x < data.player.pos.x {
                    Some(Direction::Right)
                } else if enemy.pos.x > data.player.pos.x {
                    Some(Direction::Left)
                } else {
                    None
                };
                let jump = enemy.pos.y < data.player.pos.y;
                MovementCommand::Platformer { dir, jump }
            },
            Knockback::No => MovementCommand::Velocity(V2::ZERO),
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
            Knockback::Knocked { time_remaining, just_hit: false } => {
                let new_time_remaining = time_remaining - dt;
                enemy.health.knockback = if new_time_remaining <= 0.0 {
                    Knockback::No
                } else {
                    Knockback::Knocked {
                        time_remaining: new_time_remaining,
                        just_hit: false,
                    }
                };

                MovementCommand::Velocity(V2::ZERO)
            },
        };
        if let MovementCommand::Platformer { dir: Some(dir), .. } = enemy_command {
            enemy.facing_direction = dir;
        }
        enemy.mov(&data.tilemap, enemy_command, input.dt);
    }

    ///////////////////////////////////////////////////////////////
    /* camera movement */ {
        let screen_center = v2!(
            SCREEN_WIDTH_IN_TILES as f32 / 2.0,
            SCREEN_HEIGHT_IN_TILES as f32 / 2.0,
        );
        data.camera_pos = data.player.pos - screen_center;
        clamp(
            &mut data.camera_pos.x,
            0.0,
            data.tilemap.width as f32 - SCREEN_WIDTH_IN_TILES,
        );
        clamp(
            &mut data.camera_pos.y,
            0.0,
            data.tilemap.height as f32 - SCREEN_HEIGHT_IN_TILES,
        );
    }

    // draw ////////////////////////////////////////////////////////
    render::clear(screen, Color::BLACK);

    data.tilemap.draw(screen, &data.tile_bitmaps, data.camera_pos);

    let bmp = match data.player.facing_direction {
        Direction::Right => &data.player_bmps.right,
        Direction::Left => &data.player_bmps.left,
    };
    data.player.draw(screen, bmp, data.camera_pos);

    if data.player_attack.health.hp > 0 {
        let bmp = match data.player_attack.facing_direction {
            Direction::Right => &data.player_bmps.attack_right,
            Direction::Left => &data.player_bmps.attack_left,
        };
        data.player_attack.draw(screen, bmp, data.camera_pos);
    }

    for enemy in &data.enemies {
        let bmp = match enemy.facing_direction {
            Direction::Right => &data.enemy_bmp_right,
            Direction::Left => &data.enemy_bmp_left,
        };
        match enemy.health.knockback {
            Knockback::Knocked { time_remaining, .. }
                if (time_remaining * 20.0).sin() > 0.0 => (),
            _ => enemy.draw(screen, bmp, data.camera_pos),
        }
    }

    format!(" {}", data.player)
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
        //TODO: trait for saving into file
        write_to_file("data/levels/map_00", &data.tilemap).unwrap();
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

    if input.mouse[MouseKey::LB].is_down() {
        let tile_pos = screen_pos_to_tilemap_pos(
            input.mouse.pos(),
            data.camera_pos,
            (screen.width(), screen.height()),
        );
        let _ = data.tilemap.set(tile_pos.x.trunc() as i32, tile_pos.y.trunc() as i32, Tile::Ground);
    } else if input.mouse[MouseKey::RB].is_down() {
        let tile_pos = screen_pos_to_tilemap_pos(
            input.mouse.pos(),
            data.camera_pos,
            (screen.width(), screen.height()),
        );
        let _ = data.tilemap.set(tile_pos.x.trunc() as i32, tile_pos.y.trunc() as i32, Tile::Empty);
    }

    render::clear(screen, Color::BLACK);

    data.tilemap.draw(screen, &data.tile_bitmaps, data.camera_pos);

    /* draw yellow outline */ {
        let thickness = 4;
        render::fill_rect(screen, (0, 0), (thickness, screen.height()), Color::YELLOW);
        render::fill_rect(screen, (screen.width() - thickness, 0), screen.dim(), Color::YELLOW);
        render::fill_rect(screen, (0, 0), (screen.width(), thickness), Color::YELLOW);
        render::fill_rect(screen, (0, screen.height() - thickness), screen.dim(), Color::YELLOW);
    } 

    data.font_bmp.draw_string(screen, (100, 100), "Hello, World!");

    format!("{:?}", screen.dim())
}

#[derive(Copy, Clone, Debug)]
struct Size {
    pub top_offset: f32,
    pub bottom_offset: f32,
    pub right_offset: f32,
    pub left_offset: f32,
}

impl Size {
    pub fn with_symmetric_offset(offset: V2) -> Self {
        Self {
            top_offset: offset.y,
            bottom_offset: -offset.y,
            right_offset: offset.x,
            left_offset: -offset.x,
        }
    }
}

//TODO: Size -> Rect2
#[derive(Copy, Clone, Debug)]
struct Entity {
    pub pos: V2,
    pub vel: V2,
    pub size: Size,
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
    const MAX_VELOCITY: V2 = v2!(6.0, 13.0);

    pub fn new() -> Self {
        Self {
            pos: v2!(1.5, 1.5),
            vel: v2!(0.0, 0.0),
            size: Size {
                top_offset:      0.5 - 1.0 / 9.0,
                bottom_offset: -(0.5 - 0.001),
                right_offset:    0.5 - 1.0 / 8.0,
                left_offset:   -(0.5 - 1.0 / 8.0),
            },
            health: Health { hp: 1, knockback: Knockback::No },
            facing_direction: Direction::Right,
            movement_state: MovementState::InTheAir { jumped_again: true },
        }
    }

    pub fn with_pos_health(pos: V2, hp: i32) -> Self {
        let mut entity = Self::new();
        entity.pos = pos;
        entity.health.hp = hp;
        entity
    }

    pub fn draw(&self, screen: &Bitmap, bmp: &Bitmap, camera: V2) {
        let (x0, y0) = tilemap_pos_to_screen_pos(self.pos, camera, screen.dim());
        render::draw_bmp(screen, bmp, (x0 - TILE_SIZE / 2, y0 - TILE_SIZE / 2));
    }

    //TODO: derive consts from height and length of a desired jump
    pub fn mov(&mut self, tilemap: &Tilemap, command: MovementCommand, dt: f32) {
        use std::ops::Mul;

        const HORIZONTAL_ACC: f32 = 50.0;
        const JUMP_VEL: f32 = 40.0;
        const GRAVITY: V2 = v2!(0.0, -50.0);

        fn friction(vel: f32) -> f32 { vel * -8.0 }
        fn delta_position(acc: V2, vel: V2, dt: f32) -> V2 { 0.5 * acc * dt*dt + vel * dt }
        fn delta_velocity<T>(acc: T, dt: f32) -> T where T: Mul<f32, Output=T> {
            acc * dt
        }

        let (V2 { x: mut dx, y: mut dy }, new_vel) = match command {
            MovementCommand::Velocity(new_vel) => {
                (delta_position(V2::ZERO, self.vel, dt), new_vel)
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
                        let new_vel = {
                            let new_velx = self.vel.x + delta_velocity(acc_x, dt);
                            let new_vely = if jump {
                                self.movement_state = MovementState::InTheAir {
                                    jumped_again: false,
                                };
                                JUMP_VEL
                            } else {
                                self.vel.y + delta_velocity(GRAVITY.y, dt)
                            };
                            v2!(new_velx, new_vely)
                        };
                        let acc = v2!(acc_x, 0.0);
                        (delta_position(acc, self.vel, dt), new_vel)
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
            if let Some(tile_x) = h_tilemap_collision(self, tilemap, dx) {
                let tile_x = tile_x as f32;
                self.vel.x = 0.0;
                dx = if dx > 0.0 {
                    tile_x         - self.pos.x - self.size.right_offset * 1.01
                } else {
                    (tile_x + 1.0) - self.pos.x - self.size.left_offset  * 1.01
                }
            }
        }
        self.pos.x += dx;

        if dy != 0.0 {
            if let Some(tile_y) = v_tilemap_collision(self, tilemap, dy) {
                let tile_y = tile_y as f32;
                self.vel.y = 0.0;
                dy = if dy > 0.0 {
                    tile_y         - self.pos.y - self.size.top_offset    * 1.01
                } else {
                    self.movement_state = MovementState::OnTheGround;
                    (tile_y + 1.0) - self.pos.y - self.size.bottom_offset * 1.01
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
            MovementState::OnTheGround => "_",
            MovementState::InTheAir { jumped_again: false } => "-",
            MovementState::InTheAir { jumped_again: true } => "^",
        };
        write!(f, "pos: ({:>+5.2}, {:>+5.2}), vel: ({:>+5.2}, {:>+5.2}) |{}|",
               self.pos.x, self.pos.y, self.vel.x, self.vel.y, airborne)
    }
}

#[derive(Copy, Clone, Debug)]
enum MovementCommand {
    Velocity(V2),
    Platformer { dir: Option<Direction>, jump: bool },
}

#[derive(Copy, Clone, Debug)]
struct Rect2(V2, V2);

impl Rect2 {
    #[inline(always)] pub fn right(self)  -> f32 { self.1.x }
    #[inline(always)] pub fn left(self)   -> f32 { self.0.x }
    #[inline(always)] pub fn top(self)    -> f32 { self.1.y }
    #[inline(always)] pub fn bottom(self) -> f32 { self.0.y }
    
    pub fn from_bbox(bottom_left: V2, top_right: V2) -> Self {
        Self(bottom_left, top_right)
    }

    pub fn from_center_size(center: V2, size: Size) -> Self {
        Self(
            v2!(center.x + size.left_offset, center.y + size.bottom_offset),
            v2!(center.x + size.right_offset, center.y + size.top_offset),
        )
    }
}

fn aabb_collision(rect0: Rect2, rect1: Rect2) -> bool {
    rect0.right() > rect1.left() && rect0.left() < rect1.right()
        && rect0.top() > rect1.bottom() && rect0.bottom() < rect1.top()
}

fn h_tilemap_collision(entity: &Entity, tilemap: &Tilemap, dx: f32) -> Option<i32> {
    let u_tile_y = (entity.pos.y + entity.size.top_offset   ).floor() as i32;
    let d_tile_y = (entity.pos.y + entity.size.bottom_offset).floor() as i32;

    let (offset, step_x) = if dx > 0.0 {
        (entity.size.right_offset, 1)
    } else {
        (entity.size.left_offset, -1)
    };
    let from_x = (entity.pos.x + offset).floor() as i32;
    let to_x = (entity.pos.x + offset + dx).floor() as i32;

    let mut tile_x = from_x;
    loop {
        match (tilemap.get(tile_x, u_tile_y),
               tilemap.get(tile_x, d_tile_y)) 
        {
            (Some(up), Some(dn)) if up.is_obstacle() || dn.is_obstacle() =>
                return Some(tile_x),
            (None, _   ) |
            (_   , None) =>
                return Some(tile_x),
            _ => (),
        }

        if tile_x == to_x { break }
        tile_x += step_x;
    }
    None
}

fn v_tilemap_collision(entity: &Entity, tilemap: &Tilemap, dy: f32) -> Option<i32> {
    let r_tile_x = (entity.pos.x + entity.size.right_offset).floor() as i32;
    let l_tile_x = (entity.pos.x + entity.size.left_offset ).floor() as i32;

    let (offset, step_y) = if dy > 0.0 {
        (entity.size.top_offset, 1)
    } else {
        (entity.size.bottom_offset, -1)
    };
    let from_y = (entity.pos.y + offset).floor() as i32;
    let to_y = (entity.pos.y + offset + dy).floor() as i32;

    let mut tile_y = from_y;
    loop {
        match (tilemap.get(r_tile_x, tile_y),
               tilemap.get(l_tile_x, tile_y)) 
        {
            (Some(rt), Some(lt)) if rt.is_obstacle() || lt.is_obstacle() =>
                return Some(tile_y),
            (None, _   ) |
            (_   , None) =>
                return Some(tile_y),
            _ => (),
        }

        if tile_y == to_y { break }
        tile_y += step_y;
    }
    None
}