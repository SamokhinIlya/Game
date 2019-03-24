use std::ops::{Index, IndexMut};
use crate::{
    render,
    vector::prelude::*,
    bitmap::Bitmap,
    file::prelude::*,
};

pub fn screen_pos_to_tilemap_pos(
    screen_pos: V2i,
    camera: V2f,
    screen: V2i,
    tile_size: i32,
) -> V2f {
    let x = screen_pos.x as f32 / tile_size as f32 + camera.x;
    let y = (screen.y - screen_pos.y) as f32 / tile_size as f32 + camera.y;
    v2!(x, y)
}

pub fn tilemap_pos_to_screen_pos(
    tilemap_pos: V2f,
    camera: V2f,
    screen: V2i,
    tile_size: i32,
) -> V2i {
    let x = ((tilemap_pos.x - camera.x) * tile_size as f32) as i32;
    let y = screen.y - ((tilemap_pos.y - camera.y) * tile_size as f32) as i32;
    v2!(x, y)
}

// TileInfo

pub struct TileInfo {
    pub size: i32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub screen_width_in_px: i32,
    pub screen_height_in_px: i32,
    pub bmps: [Bitmap; 1],
}

impl TileInfo {
    pub fn get_bmp(&self, tile: Tile) -> &Bitmap {
        assert!(match tile {
            Tile::Ground => true,
            _ => false,
        });
        &self.bmps[0]
    }
}

// Tile

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

// Tilemap

#[derive(Clone)]
pub struct Tilemap {
    width: i32,
    height: i32,
    map: Vec<Tile>,
}

impl Index<(i32, i32)> for Tilemap {
    type Output = Tile;
    fn index(&self, (x, y): (i32, i32)) -> &Self::Output {
        unsafe { &*self.ptr_at(x, y) }
    }
}

impl IndexMut<(i32, i32)> for Tilemap {
    fn index_mut(&mut self, (x, y): (i32, i32)) -> &mut Tile {
        unsafe { &mut *self.mut_ptr_at(x, y) }
    }
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

    fn check(&self, x: i32, y: i32) {
        assert!(
            x >= 0 && x < self.width && y >= 0 && y < self.height,
            "Tilemap index out of bounds. (w, h): {:?}, (x, y): {:?}",
            (self.width, self.height), (x, y),
        );
    }

    unsafe fn mut_ptr_at(&mut self, x: i32, y: i32) -> *mut Tile {
        self.check(x, y);
        self.map.as_mut_ptr().add((y * self.width + x) as usize)
    }

    unsafe fn ptr_at(&self, x: i32, y: i32) -> *const Tile {
        self.check(x, y);
        self.map.as_ptr().add((y * self.width + x) as usize)
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Tile> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            Some(self[(x, y)])
        } else {
            None
        }
    }

    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        assert!(
            new_width > 0 && new_height > 0,
            "Tilemap::resize: (new_width, new_height): {:?}",
            (new_width, new_height),
        );

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
        dst: &Bitmap,
        camera: V2f,
        info: &TileInfo,
    ) {
        use std::cmp::{min, max};

        let camera_i: V2i = camera.floor().into();
        let v_draw_tiles = info.screen_height.ceil() as i32;
        let h_draw_tiles = info.screen_width.ceil() as i32;

        let lower_bound = max(camera_i.y, 0);
        let upper_bound = min(camera_i.y + v_draw_tiles, self.height - 1);

        let left_bound = max(camera_i.x, 0);
        let right_bound = min(camera_i.x + h_draw_tiles, self.width - 1);

        for tile_y in lower_bound..=upper_bound {
            for tile_x in left_bound..=right_bound {
                let tile = self[(tile_x, tile_y)];
                if !tile.is_visible() { continue }

                let bmp = info.get_bmp(tile);
                // TODO: this should be checked somewhere else (on initialization maybe)
                debug_assert!(bmp.width() == bmp.height());

                let V2 { x, y } = tilemap_pos_to_screen_pos(
                    v2!(tile_x as f32, tile_y as f32),
                    camera,
                    dst.dim(),
                    info.size,
                );
                //dbg!((x, y));
                render::draw_bmp(dst, bmp, v2!(x, y - info.size));
            }
        }
    }

    pub fn draw_outline(&self, dst: &mut Bitmap, camera: V2f, info: &TileInfo) {
        let min: V2i = tilemap_pos_to_screen_pos(
            v2!(0.0, self.height as f32),
            camera,
            dst.dim(),
            info.size,
        );
        let max: V2i = tilemap_pos_to_screen_pos(
            v2!(self.width as f32, 0.0),
            camera,
            dst.dim(),
            info.size,
        );
        render::draw_rect(dst, min, max, render::Color::YELLOW, 1);
    }

    pub fn draw_grid(&self, dst: &mut Bitmap, camera: V2f, info: &TileInfo) {
        use std::cmp::{min, max};

        let camera_i: V2i = camera.floor().into();
        let v_draw_tiles = info.screen_height.ceil() as i32;
        let h_draw_tiles = info.screen_width.ceil() as i32;

        //let lower_bound = max(camera_i.y, 0);
        //let upper_bound = min(camera_i.y + v_draw_tiles, self.height - 1);
        //
        //for tile_y in lower_bound..upper_bound {
        //    let y = tile_y * info.size;
        //    let min = v2!(0, y);
        //    let max = v2!(self.width * info.size, y);
        //    render::draw_line(dst, min, max, render::Color::WHITE, 1);
        //}

        let left_bound = max(camera_i.x, 0);
        let right_bound = min(camera_i.x + h_draw_tiles, self.width - 1);

        for tile_x in left_bound..right_bound {
            let x = tile_x * info.size;
            let min = v2!(x, 0);
            let max = v2!(x, self.height * info.size);
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
        use std::mem::{size_of, uninitialized as uninit};

        let file = crate::file::read_entire_file(filepath)?;

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
        use std::mem::{size_of, uninitialized as uninit};

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
        crate::file::write_bytes_to_file(filepath, to_file.as_slice())
    }
}
