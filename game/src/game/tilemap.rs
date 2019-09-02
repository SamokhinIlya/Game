use std::ops::{Index, IndexMut};
use crate::{
    render::{self, Bitmap, screen_info::ScreenInfo},
    geom::{
        vector::prelude::*,
        matrix::Mat2,
        aabb::AABB,
    },
    file::prelude::*,
};

// TileInfo

pub struct TileInfo {
    pub size: i32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub bmps: [Bitmap; 1],
}

impl TileInfo {
    pub fn get_bmp(&self, tile: Tile) -> &Bitmap {
        use Tile::*;
        assert!(if let Ground = tile { true } else { false });
        &self.bmps[0]
    }
}

// Tile

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Tile {
    Empty = 0,
    Ground = 1,
}

impl Default for Tile {
    fn default() -> Self { Tile::Empty }
}

impl Tile {
    pub fn is_visible(self) -> bool {
        use self::Tile::*;
        match self {
            Ground => true,
            Empty => false,
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

    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }
    pub fn dim(&self) -> V2i { (self.width, self.height).into() }

    fn check(&self, x: i32, y: i32) {
        assert!(
            (0..self.width).contains(&x) && (0..self.height).contains(&y),
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
        if (0..self.width).contains(&x) && (0..self.height).contains(&y) {
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
            self.map.resize(new_width * self.height as usize, Tile::default());

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
            self.map.resize(new_width * self.height as usize, Tile::default());
        }

        if new_height != self.height {
            self.map.resize((self.width * new_height) as usize, Tile::default());
            self.height = new_height;
        }
    }

    pub fn draw(&self, dst: &Bitmap, screen_info: &ScreenInfo, tile_info: &TileInfo) {
        use std::cmp::{min, max};

        let camera_i: V2i = screen_info.camera.floor().into();
        let v_draw_tiles = tile_info.screen_height.ceil() as i32;
        let h_draw_tiles = tile_info.screen_width.ceil() as i32;

        let lower_bound = max(camera_i.y, 0);
        let upper_bound = min(camera_i.y + v_draw_tiles, self.height - 1);

        let left_bound = max(camera_i.x, 0);
        let right_bound = min(camera_i.x + h_draw_tiles, self.width - 1);

        for tile_y in lower_bound..=upper_bound {
            for tile_x in left_bound..=right_bound {
                let tile = self[(tile_x, tile_y)];
                if !tile.is_visible() {
                    continue
                }

                let V2 { x, y } = render::v2_to_screen((tile_x as f32, tile_y as f32).into(), screen_info);
                render::draw_bmp(dst, tile_info.get_bmp(tile), (x, y - tile_info.size).into());
            }
        }
    }

    pub fn draw_outline(&self, dst: &mut Bitmap, screen_info: &ScreenInfo) {
        let AABB { min, max } = render::aabb_to_screen(
            AABB { min: (0.0, 0.0).into(), max: (self.width as f32, self.height as f32).into() },
            screen_info,
        );
        render::draw_rect(dst, min, max, render::Color::YELLOW, 1);
    }

    pub fn draw_grid(&self, dst: &mut Bitmap, screen_info: &ScreenInfo, tile_info: &TileInfo) {
        use std::cmp::{min, max};
        use utils::clamp;

        let color = render::Color::GREY;
        let thickness = 1;

        let camera_i: V2i = screen_info.camera.floor().into();
        let v_draw_tiles = tile_info.screen_height.ceil() as i32;
        let h_draw_tiles = tile_info.screen_width.ceil() as i32;

        let lower_bound = max(camera_i.y, 1);
        let upper_bound = min(camera_i.y + v_draw_tiles + 1, self.height);

        for tile_y in lower_bound..upper_bound {
            let mut min = render::v2_to_screen((0., tile_y as f32).into(), screen_info);
            if !(0..dst.height()).contains(&min.y) {
                continue;
            }
            min.x = clamp(min.x, 0, dst.width());

            let mut max = render::v2_to_screen((self.width as f32, tile_y as f32).into(), screen_info);
            max.x = clamp(max.x, 0, dst.width());

            render::draw_line(dst, min, max, color, thickness);
        }

        let left_bound = max(camera_i.x, 1);
        let right_bound = min(camera_i.x + h_draw_tiles + 1, self.width);

        for tile_x in left_bound..right_bound {
            let mut min = render::v2_to_screen((tile_x as f32, self.height as f32).into(), screen_info);
            if !(0..dst.width()).contains(&min.x) {
                continue;
            }
            min.y = clamp(min.y, 0, dst.height());

            let mut max = render::v2_to_screen((tile_x as f32, 0.).into(), screen_info);
            max.y = clamp(max.y, 0, dst.height());

            render::draw_line(dst, min, max, color, thickness);
        }
    }
}

/// For saving/loading to/from file
#[repr(C)]
#[derive(Copy, Clone)]
struct TilemapSize {
    width: u32,
    height: u32,
}

impl Load for Tilemap {
    fn load(filepath: impl AsRef<Path>) -> io::Result<Self> {
        use std::mem::{size_of, uninitialized as uninit};

        let file = crate::file::read_entire_file(filepath)?;

        #[allow(clippy::cast_ptr_alignment)]
        let TilemapSize { width, height } = unsafe {
            std::ptr::read_unaligned(file.as_ptr() as *const _)
        };
        let tilemap_size = (width * height) as usize;

        let mut map = Vec::<Tile>::with_capacity(tilemap_size);
        map.resize_with(tilemap_size, || unsafe { uninit() });

        let map_as_bytes = unsafe {
            std::slice::from_raw_parts_mut(
                map.as_mut_ptr() as *mut u8,
                tilemap_size * size_of::<Tile>()
            )
        };
        map_as_bytes.copy_from_slice(&file[size_of::<TilemapSize>()..]);

        Ok(Self {
            width: width as i32,
            height: height as i32,
            map,
        })
    }
}

impl Save for Tilemap {
    fn save(&self, filepath: impl AsRef<Path>) -> io::Result<()> {
        use std::mem::{size_of, uninitialized as uninit};

        let to_file = {
            let tilemap_size = TilemapSize {
                width: self.width as u32,
                height: self.height as u32,
            };
            let tilemap_size_as_bytes = unsafe {
                std::slice::from_raw_parts(
                    &tilemap_size as *const _ as *const u8,
                    size_of::<TilemapSize>(),
                )
            };
            let tilemap_as_bytes = unsafe {
                &*(self.map.as_slice() as *const _ as *const [u8])
            };

            let filesize = size_of::<TilemapSize>() + self.map.len() / size_of::<Tile>();

            let mut bytes = Vec::<u8>::with_capacity(filesize);
            bytes.resize_with(filesize, || unsafe { uninit() });

            bytes[..size_of::<TilemapSize>()].copy_from_slice(tilemap_size_as_bytes);
            bytes[size_of::<TilemapSize>()..].copy_from_slice(tilemap_as_bytes);
            bytes
        };
        crate::file::write_bytes_to_file(filepath, to_file.as_slice())
    }
}
