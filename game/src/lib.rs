#[macro_use] mod vector;
mod game;
mod render;
mod file;

pub use game::{
    startup,
    update_and_render,
};
