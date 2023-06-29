#[derive(Copy,Clone)]
pub struct Color(pub [f32 ; 4]);
impl Color {
	pub const fn get_arr(&self) -> &[f32 ; 4] {
		&self.0
	}
	pub const fn set_alpha(self, alpha : f32) -> Color {
		let mut arr = *self.get_arr();
		arr[3] = alpha;
		Color(arr)
	}
}

pub const BLACK : Color = Color([0.0,0.0,0.0,1.0]);
pub const WHITE : Color = Color([1.0,1.0,1.0,1.0]);
pub const GRAY : Color = Color([0.5,0.5,0.5,1.0]);

pub const RED : Color = Color([1.0,0.0,0.0,1.0]);
pub const GREEN : Color = Color([0.0,1.0,0.0,1.0]);
pub const BLUE : Color = Color([0.0,0.0,1.0,1.0]);

pub const CYAN : Color = Color([0.0,1.0,1.0,1.0]);
pub const MAGENTA : Color = Color([1.0,0.0,1.0,1.0]);
pub const YELLOW : Color = Color([1.0,1.0,0.0,1.0]);

pub const ORANGE : Color = Color([1.0,0.5,0.0,1.0]);

pub const DEFAULT_COLOR : Color = WHITE;