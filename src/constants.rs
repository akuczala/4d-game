pub use std::f32::consts::{PI,E};

use crate::{components::ShapeLabel, graphics::colors::{Color, WHITE}};

pub const LINE_THICKNESS_3D: f32 = 0.01;
pub const LINE_THICKNESS_4D: f32 = 0.02;

pub const DARK_TINT: f32 = 0.01;
pub const BACKGROUND_COLOR: [f32 ; 4] = [DARK_TINT, DARK_TINT, DARK_TINT, 1.0];

pub const N_FUZZ_LINES: usize = 1000;

pub const SELECTION_COLOR: Color = WHITE.set_alpha(0.2);


pub const CUBE_LABEL_STR: &str = "Cube";
pub const COIN_LABEL_STR: &str = "Coin";