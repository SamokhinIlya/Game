extern crate core;
use platform::{
    file,
    file::Load,
    graphics::Bitmap,
    input::{Input, KBKey, MouseKey},
    memory, Opaque,
};
use render::Color;
use utils::*;

#[macro_use]
mod vector;
mod tilemap;

use crate::vector::{
    V2,
    distance_sq,
};
use crate::tilemap::{
    Tilemap,
    Tile,
    TILE_SIZE,
    SCREEN_WIDTH_IN_TILES,
    SCREEN_HEIGHT_IN_TILES,
    screen_pos_to_tilemap_pos,
    tilemap_pos_to_screen_pos,
};

/* TODO:
  - entity - entity collision

  - visuals:
    - dust cloud when changing direction
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
    pub player_attacking: bool,
    pub player_attack_counter: f32,

    pub enemies: [Entity; 1],

    pub tile_bitmaps: [Bitmap; 1],
    pub player_bmps: PlayerBmps,
    pub enemy_bmp_right: Bitmap,
    pub enemy_bmp_left: Bitmap,
}

struct PlayerBmps {
    pub right: Bitmap,
    pub left: Bitmap,
    pub attack_right: Bitmap,
    pub attack_left: Bitmap,
}

pub fn startup(_screen_width: i32, _screen_height: i32) -> *mut Opaque {
    let result = unsafe {
        memory::allocate(GameData {
            state: GameState::LevelEditor,
            tilemap: match Tilemap::load("data/levels/map_00") {
                Ok(t) => t,
                Err(_) => Tilemap::new(),
            },
            camera_pos: v2!(0.0, 0.0),
            player: Entity::with_pos_health(v2!(2.5, 2.5), 1),
            player_attack: Entity::with_pos_health(V2::new(), 0),
            player_attacking: false,
            player_attack_counter: 0.0,
            enemies: [Entity::with_pos_health(v2!(3.5, 1.5), 1); 1],
            tile_bitmaps: [Bitmap::load("data/sprites/size_64/test_tile.bmp").unwrap(); 1],
            player_bmps: PlayerBmps {
                right: Bitmap::load("data/sprites/size_64/test_player_right.bmp").unwrap(),
                left: Bitmap::load("data/sprites/size_64/test_player_left.bmp").unwrap(),
                attack_right: Bitmap::load("data/sprites/size_64/test_player_attack_right.bmp").unwrap(),
                attack_left: Bitmap::load("data/sprites/size_64/test_player_attack_left.bmp").unwrap(),
            },
            enemy_bmp_right: Bitmap::load("data/sprites/size_64/test_enemy_right.bmp").unwrap(),
            enemy_bmp_left: Bitmap::load("data/sprites/size_64/test_enemy_left.bmp").unwrap(),
        })
    };
    result as *mut Opaque
}

pub fn update_and_render(
    screen:    &mut Bitmap,
    input:     &Input,
    game_data: *mut Opaque,
) -> String {
    #[allow(clippy::cast_ptr_alignment)]
    let data: &mut GameData = unsafe {
        (game_data as *mut GameData).as_mut().unwrap()
    };
    let dt = input.dt;

    match data.state {
        GameState::Playing => playing(screen, input, data, dt),
        GameState::LevelEditor => level_editor(screen, input, data, dt),
    }
}

#[allow(clippy::useless_format)]
fn playing(
    screen: &mut Bitmap,
    input:  &Input,
    data:   &mut GameData,
    dt:     f32,
) -> String {
    if input.keyboard[KBKey::K].pressed()
    && input.keyboard[KBKey::Ctrl].is_down() {
        data.state = GameState::LevelEditor;
        render::clear(screen, Color::BLACK);
    }

    let direction = {
        use platform::input::KBKey::*;
        let (left, right, _down, _up) = (
            input.keyboard[A].is_down(),
            input.keyboard[D].is_down(),
            input.keyboard[S].is_down(),
            input.keyboard[W].is_down(),
        );
        let x = match (left, right) {
            (false, true) =>  1.0,
            (true, false) => -1.0,
            _             =>  0.0,
        };
        let _y = match (_down, _up) {
            (false, true) =>  1.0,
            (true, false) => -1.0,
            _             =>  0.0,
        };
        let y = if input.keyboard[KBKey::K].pressed() { 1.0 } else { 0.0 };
        v2!(x, y)
    };
    if data.player_attack.health > 0 {
        for enemy in &mut data.enemies {
            if aabb_collision(
                Rect2::from_center_size(data.player_attack.pos, data.player_attack.size),
                Rect2::from_center_size(enemy.pos, enemy.size),
            ) {
                enemy.health -= 1;
            }
        }

        data.player_attack_counter -= dt;
        //FIXME: try jump and attack
        data.player_attack.pos = v2!(
            data.player.pos.x + match data.player.facing_direction {
                FacingDirection::Right => 1.0,
                FacingDirection::Left => -1.0,
            },
            data.player.pos.y,
        );
        if data.player_attack_counter < 0.0 {
            data.player_attack.health = 0;
        }
    } else {
        if direction.x > 0.0 {
            data.player.facing_direction = FacingDirection::Right;
        } else if direction.x < 0.0 {
            data.player.facing_direction = FacingDirection::Left;
        }

        if input.keyboard[KBKey::J].pressed() {
            data.player_attack_counter = 0.1;

            data.player_attack.health = 1;
            data.player_attack.pos = v2!(
                data.player.pos.x + match data.player.facing_direction {
                    FacingDirection::Right => 1.0,
                    FacingDirection::Left => -1.0,
                },
                data.player.pos.y
            );
            data.player_attack.facing_direction = data.player.facing_direction;
        } 
    }
    entity_move(&mut data.player, &data.tilemap, direction, dt);

    for enemy in &mut data.enemies {
        let mut direction = v2!(0.0, 0.0);
        if distance_sq(enemy.pos, data.player.pos) >= 4.0 {
            direction.x = if enemy.pos.x < data.player.pos.x { 1.0 } else { -1.0 };
            direction.y = if enemy.pos.y < data.player.pos.y { 1.0 } else {  0.0 };
        }
        if direction.x > 0.0 {
            enemy.facing_direction = FacingDirection::Right;
        } else if direction.x < 0.0 {
            enemy.facing_direction = FacingDirection::Left;
        }
        entity_move(enemy, &data.tilemap, direction, input.dt);
    }

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

    render::clear(screen, Color::BLACK);
    data.tilemap.draw(screen, &data.tile_bitmaps, data.camera_pos);

    let bmp = match data.player.facing_direction {
        FacingDirection::Right => &data.player_bmps.right,
        FacingDirection::Left => &data.player_bmps.left,
    };
    data.player.draw(screen, bmp, data.camera_pos);
    if data.player_attack.health > 0 {
        let bmp = match data.player_attack.facing_direction {
            FacingDirection::Right => &data.player_bmps.attack_right,
            FacingDirection::Left => &data.player_bmps.attack_left,
        };
        data.player_attack.draw(screen, bmp, data.camera_pos);
    }

    for enemy in &data.enemies {
        let bmp = match enemy.facing_direction {
            FacingDirection::Right => &data.enemy_bmp_right,
            FacingDirection::Left => &data.enemy_bmp_left,
        };
        enemy.draw(screen, bmp, data.camera_pos);
    }

    format!("")
}

#[allow(clippy::useless_format)]
fn level_editor(
    screen: &mut Bitmap,
    input:  &Input,
    data:   &mut GameData,
    dt:     f32,
) -> String {
    if input.keyboard[KBKey::K].pressed()
    && input.keyboard[KBKey::Ctrl].is_down()
    {
        data.state = GameState::Playing;
    }

    if input.keyboard[KBKey::S].pressed()
    && input.keyboard[KBKey::Ctrl].is_down()
    {
        file::File::write("data/levels/map_00", &data.tilemap).unwrap();
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
            (screen.width, screen.height),
        );
        data.tilemap.set(tile_pos.x.trunc() as i32, tile_pos.y.trunc() as i32, Tile::Ground);
    }
    else if input.mouse[MouseKey::RB].is_down() {
        let tile_pos = screen_pos_to_tilemap_pos(
            input.mouse.pos(),
            data.camera_pos,
            (screen.width, screen.height),
        );
        data.tilemap.set(tile_pos.x.trunc() as i32, tile_pos.y.trunc() as i32, Tile::Empty);
    }

    render::clear(screen, Color::BLACK);
    data.tilemap.draw(screen, &data.tile_bitmaps, data.camera_pos);

    let thickness = 4;
    render::fill_rect(screen, 0, 0, thickness, screen.height, Color::YELLOW);
    render::fill_rect(screen, screen.width - thickness, 0, screen.width, screen.height, Color::YELLOW);
    render::fill_rect(screen, 0, 0, screen.width, thickness, Color::YELLOW);
    render::fill_rect(screen, 0, screen.height - thickness, screen.width, screen.height, Color::YELLOW);

    format!("")
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

struct Entity {
    pub pos: V2,
    pub vel: V2,
    pub size: Size,
    pub facing_direction: FacingDirection,
    pub health: i32,
    pub on_the_ground: bool,
}

#[derive(Copy, Clone)]
enum FacingDirection {
    Left,
    Right,
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
            health: 1,
            facing_direction: FacingDirection::Right,
            on_the_ground: false,
        }
    }

    pub fn with_pos_health(pos: V2, health: i32) -> Self {
        let mut entity = Self::new();
        entity.pos = pos;
        entity.health = health;
        entity
    }

    pub fn draw(&self, screen: &Bitmap, bmp: &Bitmap, camera: V2) {
        if self.health <= 0 {
            return
        }
        let (x0, y0) = tilemap_pos_to_screen_pos(self.pos, camera, screen.dim());
        render::draw_bmp(screen, bmp, x0 - TILE_SIZE / 2, y0 - TILE_SIZE / 2);
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let airborne = if self.on_the_ground { "_" } else { "^" };
        write!(f, "({}, {}) |{}|", self.pos.x, self.pos.y, airborne)
    }
}

//TODO: derive consts from height and length of a desired jump
fn entity_move(entity: &mut Entity, tilemap: &Tilemap, direction: V2, dt: f32) {
    assert!(direction.x >= -1.0 && direction.x <= 1.0
         && direction.y >= -1.0 && direction.y <= 1.0, "direction = {}", direction);

    const ACCELERATION: f32 = 50.0;

    let acc_x = if entity.on_the_ground {
        let friction = -8.0 * entity.vel.x;
        direction.x * ACCELERATION + friction
    } else {
        let air_movement_penalty = -direction.x * ACCELERATION * 0.8;
        direction.x * ACCELERATION + air_movement_penalty
    };
    let mut dx = 0.5 * acc_x * dt*dt + entity.vel.x * dt;
    entity.vel.x += acc_x * dt;
    clamp(&mut entity.vel.x, -Entity::MAX_VELOCITY.x, Entity::MAX_VELOCITY.x);

    if dx != 0.0 {
        if let Some(tile_x) = h_tilemap_collision(entity, tilemap, dx) {
            entity.vel.x = 0.0;
            dx = if dx > 0.0 {
                tile_x       as f32 - entity.pos.x - entity.size.right_offset * 1.01
            } else {
                (tile_x + 1) as f32 - entity.pos.x - entity.size.left_offset  * 1.01
            }
        }
    }
    entity.pos.x += dx;

    if entity.on_the_ground && direction.y > 0.0 {
        entity.vel.y = 10.0;
        entity.on_the_ground = false;
    } else {
        const GRAVITY: f32 = -40.0;
        entity.vel.y += 0.5 * GRAVITY * dt;
    }
    clamp(&mut entity.vel.y, -Entity::MAX_VELOCITY.y, Entity::MAX_VELOCITY.y);

    let mut dy = entity.vel.y * dt;
    if dy != 0.0 {
        if let Some(tile_y) = v_tilemap_collision(entity, tilemap, dy) {
            dy = if dy > 0.0 {
                entity.vel.y = 0.0;
                tile_y       as f32 - entity.pos.y - entity.size.top_offset    * 1.01
            } else {
                entity.on_the_ground = true;
                (tile_y + 1) as f32 - entity.pos.y - entity.size.bottom_offset * 1.01
            }
        } else if entity.on_the_ground {
            entity.on_the_ground = false;
        }
    }
    entity.pos.y += dy;
}

fn entity_collision(e0: &Entity, e1: &Entity) -> bool {
    aabb_collision(
        Rect2::from_center_size(e0.pos, e0.size),
        Rect2::from_center_size(e1.pos, e1.size),
    )
}

#[derive(Copy, Clone, Debug)]
struct Rect2(V2, V2);

impl Rect2 {
    pub fn from_bb(v0: V2, v1: V2) -> Self {
        Rect2(v0, v1)
    }

    pub fn from_center_size(center: V2, size: Size) -> Self {
        Rect2(
            v2!(center.x + size.left_offset, center.y + size.bottom_offset),
            v2!(center.x + size.right_offset, center.y + size.top_offset),
        )
    }
    
    #[inline]
    pub fn right(self)  -> f32 { self.1.x }
    #[inline]
    pub fn left(self)   -> f32 { self.0.x }
    #[inline]
    pub fn top(self)    -> f32 { self.1.y }
    #[inline]
    pub fn bottom(self) -> f32 { self.0.y }
}

fn aabb_collision(rect0: Rect2, rect1: Rect2) -> bool {
    rect0.right() > rect1.left()
    && rect0.left() < rect1.right()
    && rect0.top() > rect1.bottom()
    && rect0.bottom() < rect1.top()
}

fn h_tilemap_collision(
    entity: &Entity,
    tilemap: &Tilemap,
    dx: f32,
) -> Option<i32> {
    let u_tile_y = (entity.pos.y + entity.size.top_offset   ).floor() as i32;
    let d_tile_y = (entity.pos.y + entity.size.bottom_offset).floor() as i32;

    let (offset                  , step_x) = if dx > 0.0 {
        (entity.size.right_offset, 1     )
    } else {
        (entity.size.left_offset , -1    )
    };
    let from_x = (entity.pos.x + offset).floor() as i32;
    let to_x =   (entity.pos.x + offset + dx).floor() as i32;

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

fn v_tilemap_collision(
    entity: &Entity,
    tilemap: &Tilemap,
    dy: f32,
) -> Option<i32> {
    let r_tile_x = (entity.pos.x + entity.size.right_offset).floor() as i32;
    let l_tile_x = (entity.pos.x + entity.size.left_offset ).floor() as i32;

    let (offset                   , step_y) = if dy > 0.0 {
        (entity.size.top_offset   , 1     )
    } else {
        (entity.size.bottom_offset, -1    )
    };
    let from_y = (entity.pos.y + offset).floor() as i32;
    let to_y =   (entity.pos.y + offset + dy).floor() as i32;

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