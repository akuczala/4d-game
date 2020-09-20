use specs::{Component, VecStorage};

use crate::vector::{VectorTrait,MatrixTrait,Field,
	VecIndex,rotation_matrix,Rotatable,Translatable};
use crate::geometry::{Plane};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Camera<V>
where V : VectorTrait
{
	pub pos : V,
	pub frame : V::M,
	pub heading : V::M,
	pub plane : Plane<V>,

}
impl<V> Camera<V>
where V : VectorTrait
{
	const SPEED : Field = 1.5;
	const ANG_SPEED : Field = 1.5*3.14159/3.0;
	const MAX_TILT : Field = 0.99;

	pub fn new(pos : V) -> Camera<V> {
		Camera{
			pos,
			frame : V::M::id(),
			heading : V::M::id(),
			plane : Plane{normal : V::one_hot(-1), threshold : V::one_hot(-1).dot(pos)},

		}
	}
	pub fn look_at(&mut self, point : &V) {
		//self.frame = rotation_matrix(V::one_hot(-1),*point - self.pos,None).transpose();
		self.frame = rotation_matrix(*point - self.pos, V::one_hot(-1), None);
		self.update();
	}
	pub fn slide(&mut self, direction : V, time : Field) {
		self.translate(self.get_slide_dpos(direction, time));
	}
	pub fn get_slide_dpos(&self, direction : V, time : Field) -> V {
		direction.normalize()*Self::SPEED*time
	}
	pub fn spin(&mut self, axis1 : VecIndex, axis2 : VecIndex, speed_mult : Field) {
		let rot = rotation_matrix(self.frame[axis1],self.frame[axis2],Some(speed_mult*Self::ANG_SPEED));
		self.frame = self.frame.dot(rot);
		self.heading = self.heading.dot(rot);
		self.update();
	}
	//heading-based rotation affecting both frame and heading
	pub fn turn(&mut self, axis1 : VecIndex, axis2 : VecIndex, speed_mult : Field) {
		let rot = rotation_matrix(self.heading[axis1],self.heading[axis2],Some(speed_mult*Self::ANG_SPEED));
		self.frame = self.frame.dot(rot);
		self.heading = self.heading.dot(rot);
		self.update();
	}
	//heading-based rotation affecting only frame
	pub fn tilt(&mut self, axis1 : VecIndex, axis2 : VecIndex, speed_mult : Field) {
		
		let dot = self.heading[axis1].dot(self.frame[axis2]); // get projection of frame axis along heading axis

		if dot*speed_mult < 0. ||  dot.abs() < Self::MAX_TILT { //rotate if tilting direction is opposite projection or if < max tilt
		//if true {
			let rot = rotation_matrix(self.frame[axis1],self.frame[axis2],Some(speed_mult*Self::ANG_SPEED));
			self.frame = self.frame.dot(rot);

			self.update();
		}
		
	}
	pub fn update_plane(&mut self) {
		self.plane = Plane{
			normal : self.frame[-1],
			threshold : self.frame[-1].dot(self.pos)
		}
	}
	// pub fn update_heading(&mut self) {
	// 	self.heading = self.frame;
	// }
	pub fn update(&mut self) {
		//self.update_heading();
		self.update_plane();
	}
}

impl<V : VectorTrait> Rotatable<V> for Camera<V> {
	fn get_frame(&self) -> V::M {
		self.frame
	}
	fn set_frame(&mut self, frame : V::M){
		self.frame = frame;
		self.update();
	}
}
impl <V: VectorTrait> Translatable<V> for Camera<V> {
	fn get_pos(&self) -> V {
		self.pos
	}
	fn set_pos(&mut self, new_pos : V) {
		self.pos = new_pos;
		self.update();
	}
}


