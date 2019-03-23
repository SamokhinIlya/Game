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

//FIXME: variable
pub const TILE_SIZE: i32 = 64;
//FIXME: these should be derived from TILE_SIZE and screen_size
pub const H_DRAW_TILES: i32 = 15;
pub const V_DRAW_TILES: i32 = 9;
pub const SCREEN_WIDTH_IN_TILES: f32 = 15.0;
pub const SCREEN_HEIGHT_IN_TILES: f32 = 8.4375;

pub fn screen_pos_to_tilemap_pos(
    screen_pos: V2i,
    camera: V2f,
    screen: V2i,
) -> V2f {
    v2!(
        screen_pos.x as f32 / TILE_SIZE as f32 + camera.x,
        (screen.y - screen_pos.y) as f32 / TILE_SIZE as f32 + camera.y
    )
}

pub fn tilemap_pos_to_screen_pos(
    tilemap_pos: V2f,
    camera: V2f,
    screen: V2i,
) -> V2i {
    v2!(
        ((tilemap_pos.x - camera.x) * TILE_SIZE as f32) as i32,
        screen.y - ((tilemap_pos.y - camera.y) * TILE_SIZE as f32) as i32,
    )
}

#[derive(Copy, Clone, Debug)]
pub enum Tile {
    Empty = 0,
    Ground = 1,
}

impl Default for Tile {
    fn default() -> Self { Tile::Empty }
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
    width: i32,
    height: i32,
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

    #[inline(always)]
    pub fn width(&self) -> i32 { self.width }

    #[inline(always)]
    pub fn height(&self) -> i32 { self.height }

    #[inline(always)]
    pub fn dim(&self) -> V2i { v2!(self.width, self.height) }

    //TODO: trait Index
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

    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        assert!(new_width > 0, new_height > 0);

        let old_width = self.width as usize;
        self.width = new_width;
        let new_width = new_width as usize;
        if new_width > old_width {
            let dwidth = new_width - old_width;
            self.map.resize(new_width * self.height as usize, Default::default());

            let mut cursor = old_width;
            while cursor < self.map.len() {
                self.map[cursor..].rotate_right(dwidth);
                cursor += new_width;
            }
        } else if new_width < old_width {
            let dwidth = old_width - new_width;

            let mut cursor = new_width;
            while cursor < self.map.len() {
                self.map[cursor..].rotate_left(dwidth);
                cursor += new_width;
            }
            self.map.resize(new_width * self.height as usize, Default::default());
        }

        if new_height != self.height {
            self.map.resize((self.width * new_height) as usize, Default::default());
            self.height = new_height;
        }
    }

    pub fn draw(
        &self,
        dst_bmp: &Bitmap,
        tile_bitmaps: &[Bitmap],
        camera: V2f,
    ) {
        for tile_y in (0..=V_DRAW_TILES)
            .map(|y| camera.y.trunc() as i32 + y)
            .filter(|&tile_y| tile_y >= 0 && tile_y < self.height)
        {
            for tile_x in (0..=H_DRAW_TILES)
                .map(|x| camera.x.trunc() as i32 + x)
                .filter(|&tile_x| tile_x >= 0 && tile_x < self.width)
            {
                let tile = unsafe { self.get_unchecked(tile_x, tile_y) };
                if !tile.is_visible() { continue }

                let tile_bmp: &Bitmap = get_tile_bmp(tile_bitmaps, tile);
                // TODO: this should be checked somewhere else (on initialization maybe)
                debug_assert!(tile_bmp.width() == tile_bmp.height());

                let V2 { x, y } = tilemap_pos_to_screen_pos(
                    v2!(tile_x as f32, tile_y as f32),
                    camera,
                    dst_bmp.dim()
                );
                render::draw_bmp(dst_bmp, tile_bmp, v2!(x, y - TILE_SIZE));
            }
        }
    }

    pub fn draw_outline(&self, dst: &mut Bitmap, camera: V2f) {
        let min: V2i = tilemap_pos_to_screen_pos(
            v2!(0.0, self.height as f32),
            camera,
            dst.dim(),
        );
        let max: V2i = tilemap_pos_to_screen_pos(
            v2!(self.width as f32, 0.0),
            camera,
            dst.dim(),
        );
        render::draw_rect(dst, min, max, render::Color::YELLOW, 1);
    }

    pub fn draw_grid(&self, dst: &mut Bitmap, camera: V2f) {
        for tile_y in (0..=V_DRAW_TILES)
            .map(|y| camera.y.trunc() as i32 + y)
            .filter(|&tile_y| tile_y >= 0 && tile_y < self.width)
        {
            let y = tile_y * TILE_SIZE;
            let min = v2!(0, y);
            let max = v2!(self.width * TILE_SIZE, y);
            render::draw_line(dst, min, max, render::Color::WHITE, 1);
        }

        for tile_x in (0..=H_DRAW_TILES)
            .map(|x| camera.x.trunc() as i32 + x)
            .filter(|&tile_x| tile_x >= 0 && tile_x < self.width)
        {
            let x = tile_x * TILE_SIZE;
            let min = v2!(x, 0);
            let max = v2!(x, self.height * TILE_SIZE);
            render::draw_line(dst, min, max, render::Color::WHITE, 1);
        }
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