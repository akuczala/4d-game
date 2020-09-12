use specs::{Component,
            System, VecStorage, WriteStorage};

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
	pub heading : V,
	pub plane : Plane<V>,

}
impl<V> Camera<V>
where V : VectorTrait
{
	const SPEED : Field = 1.5;
	const ANG_SPEED : Field = 1.5*3.14159/3.0;

	pub fn new(pos : V) -> Camera<V> {
		Camera{
			pos,
			frame : V::M::id(),
			heading : V::one_hot(-1),
			plane : Plane{normal : V::one_hot(-1), threshold : V::one_hot(-1).dot(pos)},

		}
	}
	pub fn look_at(&mut self, point : &V) {
		//self.frame = rotation_matrix(V::one_hot(-1),*point - self.pos,None).transpose();
		self.frame = rotation_matrix(*point - self.pos, V::one_hot(-1), None);
		self.update();
	}
	pub fn slide(&mut self, direction : V, time : Field) {
		self.translate(direction.normalize()*Self::SPEED*time);
	}
	pub fn spin(&mut self, axis1 : VecIndex, axis2 : VecIndex, speed_mult : Field) {
		self.rotate(axis1,axis2,speed_mult*Self::ANG_SPEED);
	}
	pub fn update_plane(&mut self) {
		self.plane = Plane{
			normal : self.frame[-1],
			threshold : self.frame[-1].dot(self.pos)
		}
	}
	pub fn update_heading(&mut self) {
		self.heading = self.frame[-1];
	}
	pub fn update(&mut self) {
		self.update_heading();
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


