pub use std::f32::consts::{E, PI};

use crate::{components::ShapeLabel, draw::ViewportShape, graphics::colors::*, vector::Field};

pub const HALF_PI: Field = PI * HALF;
pub const ZERO: Field = 0.0;
pub const HALF: Field = 0.5;

pub const LINE_THICKNESS_3D: f32 = 0.01;
pub const LINE_THICKNESS_4D: f32 = 0.02;

pub const DARK_TINT: f32 = 0.01;
pub const BACKGROUND_COLOR: [f32; 4] = [DARK_TINT, DARK_TINT, DARK_TINT, 1.0];

pub const CARDINAL_COLORS: [Color; 8] = [RED, GREEN, BLUE, CYAN, MAGENTA, YELLOW, ORANGE, WHITE];
pub const AXES_COLORS: [Color; 4] = [RED, GREEN, CYAN, MAGENTA];

pub const N_FUZZ_LINES: usize = 500;
pub const N_SKY_FUZZ_LINES: usize = 1000;
pub const N_HORIZON_FUZZ_LINES: usize = 500;

pub const SKY_DISTANCE: Field = 1e4;
pub const STAR_SIZE: Field = 100.0;
pub const SKY_FUZZ_SIZE: Field = 100.0;

pub const SELECTION_COLOR: Color = WHITE.set_alpha(0.2);
pub const CURSOR_COLOR: Color = WHITE;

pub const CUBE_LABEL_STR: &str = "Cube";
pub const COIN_LABEL_STR: &str = "Coin";

pub const FOCAL: Field = 1.0;

pub const Z0: Field = ZERO;

pub const SMALL_Z: Field = 0.001;
pub const Z_NEAR: Field = 0.1;

pub const CLIP_SPHERE_RADIUS: Field = 0.5;

pub const VIEWPORT_SHAPE: ViewportShape = ViewportShape::Cylinder;

pub const FACE_SCALE: Field = 0.95;

pub const PLAYER_COLLIDE_DISTANCE: Field = 0.2;
