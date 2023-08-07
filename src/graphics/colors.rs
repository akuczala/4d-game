use serde::{Deserialize, Serialize};

use crate::vector::scalar_linterp;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Color(pub [f32; 4]);
impl Color {
    pub const fn get_arr(&self) -> &[f32; 4] {
        &self.0
    }
    pub fn from_iter<I: Iterator<Item = f32>>(iter: I) -> Self {
        Color(iter.collect::<Vec<f32>>().try_into().unwrap())
    }
    pub const fn set_alpha(self, alpha: f32) -> Color {
        let mut arr = *self.get_arr();
        arr[3] = alpha;
        Color(arr)
    }
}

pub fn blend(color_1: Color, color_2: Color, t: f32) -> Color {
    Color::from_iter(
        color_1
            .0
            .iter()
            .zip(color_2.0.iter())
            .map(|(c1, c2)| scalar_linterp(*c1, *c2, t)),
    )
}

pub const BLACK: Color = Color([0.0, 0.0, 0.0, 1.0]);
pub const WHITE: Color = Color([1.0, 1.0, 1.0, 1.0]);
pub const GRAY: Color = Color([0.5, 0.5, 0.5, 1.0]);

pub const RED: Color = Color([1.0, 0.0, 0.0, 1.0]);
pub const GREEN: Color = Color([0.0, 1.0, 0.0, 1.0]);
pub const BLUE: Color = Color([0.0, 0.0, 1.0, 1.0]);

pub const CYAN: Color = Color([0.0, 1.0, 1.0, 1.0]);
pub const MAGENTA: Color = Color([1.0, 0.0, 1.0, 1.0]);
pub const YELLOW: Color = Color([1.0, 1.0, 0.0, 1.0]);

pub const ORANGE: Color = Color([1.0, 0.5, 0.0, 1.0]);

pub const DEFAULT_COLOR: Color = WHITE;
