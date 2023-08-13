pub use std::f32::consts::{E, PI};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_3};

use crate::{graphics::colors::*, vector::Field};

pub const HALF_PI: Field = FRAC_PI_2;
pub const ZERO: Field = 0.0;
pub const HALF: Field = 0.5;

pub const DARK_TINT: f32 = 0.01;
pub const DARK_BACKGROUND_COLOR: [f32; 4] = [DARK_TINT, DARK_TINT, DARK_TINT, 1.0];
pub const BACKGROUND_COLOR: [f32; 4] = DARK_BACKGROUND_COLOR;

pub const CARDINAL_COLORS: [Color; 8] = [RED, GREEN, BLUE, CYAN, MAGENTA, YELLOW, ORANGE, WHITE];
pub const AXES_COLORS: [Color; 4] = [RED, GREEN, CYAN, MAGENTA];

pub const SKY_DISTANCE: Field = 1e4;
pub const STAR_SIZE: Field = 100.0;
pub const SKY_FUZZ_SIZE: Field = 100.0;

pub const SELECTION_COLOR: Color = WHITE.set_alpha(0.2);
pub const CURSOR_COLOR: Color = WHITE;

pub const CUBE_LABEL_STR: &str = "Cube";
pub const COIN_LABEL_STR: &str = "Coin";
pub const ONE_SIDED_FACE_LABEL_STR: &str = "OneSidedFace";
pub const TWO_SIDED_FACE_LABEL_STR: &str = "TwoSidedFace";
pub const INVERTED_CUBE_LABEL_STR: &str = "InvertedCube";

pub const Z0: Field = ZERO;

pub const SMALL_Z: Field = 0.001;
pub const Z_NEAR: Field = 0.1;

pub const PLAYER_COLLIDE_DISTANCE: Field = 0.2;

pub const SPEED: Field = 1.5;
pub const ANG_SPEED: Field = 1.5 * FRAC_PI_3;
pub const MAX_TILT: Field = 0.99;

pub const MAX_TARGET_DIST: Field = 10.;

pub const FRAME_MS: u64 = 16;

pub const CONFIG_FILE_PATH_STR: &str = "./4d_config.toml";
pub const DEFAULT_SAVELOAD_PATH_STR: &str = "./";
