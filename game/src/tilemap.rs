use core::{ ptr, mem };
use platform::{
    graphics::Bitmap,
    file::{File, Load, LoadErr},
};
use crate::vector::V2;

pub const TILE_SIZE: i32 = 64;
//FIXME: these should be derived from TILE_SIZE and screen_size
pub const H_DRAW_TILES: i32 = 16;
pub const V_DRAW_TILES: i32 = 11;
pub const SCREEN_WIDTH_IN_TILES: f32 = 15.0;
pub const SCREEN_HEIGHT_IN_TILES: f32 = 8.45; //FIXME: not quite

pub fn screen_pos_to_tilemap_pos(
    screen_pos: (i32, i32),
    camera: V2,
    screen: (i32, i32),
) -> V2 {
    v2!(screen_pos.0 as f32 / TILE_SIZE as f32 + camera.x,
        (screen.1 - screen_pos.1) as f32 / TILE_SIZE as f32 + camera.y)
}

pub fn tilemap_pos_to_screen_pos(
    tilemap_pos: V2,
    camera: V2,
    screen: (i32, i32),
) -> (i32, i32) {
    (
        ((tilemap_pos.x - camera.x) * TILE_SIZE as f32) as i32,
        screen.1 - ((tilemap_pos.y - camera.y) * TILE_SIZE as f32) as i32,
    )
}

#[derive(Copy, Clone)]
pub enum Tile {
    Empty = 0,
    Ground = 1,
}

impl Tile {
    pub fn is_obstacle(self) -> bool {
        use self::Tile::*;
        match self {
            Ground => true,
            _ => false,
        }
    }

    pub fn is_visible(self) -> bool {
        use self::Tile::*;
        match self {
            Ground => true,
            _ => false,
        }
    }
}

const TILEMAP_WIDTH: usize = 16 * 2;
const TILEMAP_HEIGHT: usize = 9 * 2;

//TODO: variable size
#[derive(Clone)]
pub struct Tilemap {
    pub width: i32,
    pub height: i32,
    pub map: [[Tile; TILEMAP_WIDTH]; TILEMAP_HEIGHT],
}

impl Tilemap {
    pub fn new() -> Self {
        Self {
            width: TILEMAP_WIDTH as i32,
            height: TILEMAP_HEIGHT as i32,
            map: [ [Tile::Empty; TILEMAP_WIDTH]; TILEMAP_HEIGHT ],
        }
    }

    pub fn get_unchecked(&self, x: i32, y: i32) -> Tile {
        debug_assert!(x >= 0 && x < self.width
                   && y >= 0 && y < self.height);
        self.map[y as usize][x as usize]
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Tile> {
        if x >= 0 && x < self.width
        && y >= 0 && y < self.height {
            Some(self.map[y as usize][x as usize])
        } else {
            None
        }
    }

    pub fn set_unchecked(&mut self, x: i32, y: i32, tile: Tile) {
        debug_assert!(x >= 0 && x < self.width
                   && y >= 0 && y < self.height);
        self.map[y as usize][x as usize] = tile;
    }

    pub fn set(&mut self, x: i32, y: i32, tile: Tile) {
        if x >= 0 && x < self.width
        && y >= 0 && y < self.height {
            self.map[y as usize][x as usize] = tile;
        }
    }

    pub fn draw(
        &self,
        dst_bmp: &Bitmap,
        tile_bitmaps: &[Bitmap],
        camera: V2,
    ) {
        for y in 0..V_DRAW_TILES {
            let tile_y = camera.y.trunc() as i32 + y;
            if tile_y < 0 { continue }
            if tile_y >= self.height { break }

            for x in 0..H_DRAW_TILES {
                let tile_x = camera.x.trunc() as i32 + x;
                if tile_x < 0 { continue }
                if tile_x >= self.width { break }

                let tile = self.get_unchecked(tile_x, tile_y);
                if !tile.is_visible() { continue }

                let tile_bmp: &Bitmap = get_tile_bmp(tile_bitmaps, tile);
                debug_assert!(tile_bmp.width == tile_bmp.height);

                let (x0, y0) = tilemap_pos_to_screen_pos(
                    v2!(tile_x as f32, tile_y as f32),
                    camera,
                    (dst_bmp.width, dst_bmp.height)
                );
                render::draw_bmp(dst_bmp, tile_bmp, x0, y0 - TILE_SIZE);
            }
        }
    }
}

impl Load for Tilemap {
    fn load(filepath: &str) -> Result<Self, LoadErr> {
        match File::read(filepath) {
            Ok(file) => if file.size as usize == mem::size_of::<Tilemap>() {
                Ok(Tilemap::from(file))
            } else {
                Err(LoadErr::NotValid)
            },
            Err(err) => Err(LoadErr::FileErr(err)),
        }
    }
}

impl core::convert::From<File> for Tilemap {
    fn from(file: File) -> Self {
        #[allow(clippy::cast_ptr_alignment)]
        unsafe { ptr::read(file.data as *mut Self) }
    }
}

pub fn get_tile_bmp(tile_bitmaps: &[Bitmap], tile: Tile) -> &Bitmap {
    assert!(match tile {
        Tile::Ground => true,
        _ => false,
    });
    &tile_bitmaps[0]
}