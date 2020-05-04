#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
#![allow(clippy::items_after_statements)]

mod game;
mod render;
mod file;
mod geom;

pub use game::{
    startup,
    update_and_render,
};
