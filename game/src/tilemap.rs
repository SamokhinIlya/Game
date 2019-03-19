use std::{
    ptr,
    mem::size_of,
};
use std::mem::uninitialized as uninit;
use crate::{
    render,
    vector::{V2, V2f, V2i},
    bitmap::Bitmap,
    //TODO: prelude?
    file::*,
};

pub const TILE_SIZE: i32 = 64;
//FIXME: these should be derived from TILE_SIZE and screen_size
pub const H_DRAW_TILES: i32 = 15;
pub const V_DRAW_TILES: i32 = 9;
pub const SCREEN_WIDTH_IN_TILES: f32 = 15.0;
pub const SCREEN_HEIGHT_IN_TILES: f32 = 8.4375;

//TODO: (i32, i32) to V2i
pub fn screen_pos_to_tilemap_pos(
    screen_pos: (i32, i32),
    camera: V2f,
    screen: (i32, i32),
) -> V2f {
    v2!(
        screen_pos.0 as f32 / TILE_SIZE as f32 + camera.x,
        (screen.1 - screen_pos.1) as f32 / TILE_SIZE as f32 + camera.y
    )
}

pub fn tilemap_pos_to_screen_pos(
    tilemap_pos: V2f,
    camera: V2f,
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

#[derive(Clone)]
pub struct Tilemap {
    pub width: i32,
    pub height: i32,
    pub map: Vec<Tile>,
}

impl Tilemap {
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width > 0, height > 0);
        Self {
            width,
            height,
            map: vec![Tile::Empty; (width * height) as usize],
        }
    }

    pub unsafe fn get_unchecked(&self, x: i32, y: i32) -> Tile {
        debug_assert!(x >= 0 && x < self.width, y >= 0 && y < self.height);
        *self.map.get_unchecked((y * self.width + x) as usize)
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Tile> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            Some(unsafe { self.get_unchecked(x, y) })
        } else {
            None
        }
    }

    pub unsafe fn set_unchecked(&mut self, x: i32, y: i32, tile: Tile) {
        debug_assert!(x >= 0 && x < self.width, y >= 0 && y < self.height);
        *self.map.get_unchecked_mut((y * self.width + x) as usize) = tile;
    }

    pub fn set(&mut self, x: i32, y: i32, tile: Tile) -> Result<(), ()> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            unsafe { self.set_unchecked(x, y, tile) }
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn draw(
        &self,
        dst_bmp: &Bitmap,
        tile_bitmaps: &[Bitmap],
        camera: V2f,
    ) {
        for y in 0..=V_DRAW_TILES {
            let tile_y = camera.y.trunc() as i32 + y;
            if tile_y < 0 { continue }
            if tile_y >= self.height { break }

            for x in 0..=H_DRAW_TILES {
                let tile_x = camera.x.trunc() as i32 + x;
                if tile_x < 0 { continue }
                if tile_x >= self.width { break }

                let tile = unsafe { self.get_unchecked(tile_x, tile_y) };
                if !tile.is_visible() { continue }

                let tile_bmp: &Bitmap = get_tile_bmp(tile_bitmaps, tile);
                // TODO: this should be checked somewhere else (on initialization maybe)
                debug_assert!(tile_bmp.width() == tile_bmp.height());

                let (x0, y0) = tilemap_pos_to_screen_pos(
                    v2!(tile_x as f32, tile_y as f32),
                    camera,
                    (dst_bmp.width(), dst_bmp.height())
                );
                render::draw_bmp(dst_bmp, tile_bmp, (x0, y0 - TILE_SIZE));
            }
        }
    }

    pub fn draw_outline(&self, dst: &mut Bitmap, camera: V2f) {
        let min: V2i = tilemap_pos_to_screen_pos(
            v2!(0.0, self.height as f32),
            camera,
            dst.dim(),
        ).into();
        let max: V2i = tilemap_pos_to_screen_pos(
            v2!(self.width as f32, 0.0),
            camera,
            dst.dim(),
        ).into();
        let thickness = 1;
        render::draw_rect(dst, min, max, thickness, render::Color::YELLOW);
    }
}

/// For saving/loading to/from file
#[derive(Copy, Clone)]
#[repr(packed)]
struct TilemapSize {
    width: u32,
    height: u32,
}

impl Load for Tilemap {
    fn load<P>(filepath: P) -> io::Result<Self>
        where P: AsRef<Path>
    {
        let file = read_entire_file(filepath)?;

        #[allow(clippy::cast_ptr_alignment)]
        let TilemapSize { width, height } = unsafe {
            *(&file[..size_of::<TilemapSize>()] as *const _ as *const TilemapSize)
        };
        let tilemap_size = (width * height) as usize;

        let mut map = Vec::<Tile>::with_capacity(tilemap_size);
        map.resize_with(tilemap_size, || unsafe { uninit() });

        let map_bytes = unsafe {
            std::slice::from_raw_parts_mut(
                map.as_mut_ptr() as *mut u8,
                tilemap_size * size_of::<Tile>()
            )
        };
        map_bytes.copy_from_slice(&file[size_of::<TilemapSize>()..]);

        Ok(Self {
            width: width as i32,
            height: width as i32,
            map,
        })
    }
}

impl Save for Tilemap {
    fn save<P>(&self, filepath: P) -> io::Result<()>
        where P: AsRef<Path>
    {
        let to_file = {
            let tilemap_size = TilemapSize {
                width: self.width as u32,
                height: self.height as u32,
            };
            let tilemap_size_bytes = unsafe {
                std::slice::from_raw_parts(
                    &tilemap_size as *const _ as *const u8,
                    size_of::<TilemapSize>(),
                )
            };
            let tilemap_bytes = unsafe {
                &*(self.map.as_slice() as *const _ as *const [u8])
            };
            let filesize = size_of::<TilemapSize>() + self.map.len() / size_of::<Tile>();

            let mut bytes = Vec::<u8>::with_capacity(filesize);
            bytes.resize_with(filesize, || unsafe { uninit() });

            bytes[..size_of::<TilemapSize>()].copy_from_slice(tilemap_size_bytes);
            bytes[size_of::<TilemapSize>()..].copy_from_slice(tilemap_bytes);
            bytes
        };
        write_bytes_to_file(filepath, to_file.as_slice())
    }
}

pub fn get_tile_bmp(tile_bitmaps: &[Bitmap], tile: Tile) -> &Bitmap {
    assert!(match tile {
        Tile::Ground => true,
        _ => false,
    });
    &tile_bitmaps[0]
}