use crate::geom::{
    matrix::Mat2,
    vector::prelude::*,
};

pub struct ScreenInfo {
    pub width: i32,
    pub height: i32,

    pub scale: i32,
    pub game_to_screen_matrix: Mat2::<f32>,
    pub screen_to_game_matrix: Mat2::<f32>,

    pub camera: V2f,
}