#![warn(clippy::pedantic)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::module_name_repetitions)]

#[macro_use] mod vector;
mod game;
mod render;
mod file;

pub use game::{
    startup,
    update_and_render,
};
