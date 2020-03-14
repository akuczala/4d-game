use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex,rotation_matrix};
use crate::geometry::{Plane};

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
	const SPEED : Field = 2.0;
	const ANG_SPEED : Field = 2.0*3.14159/3.0;
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
		self.pos = self.pos + direction.normalize()*Self::SPEED*time;
		self.update();
	}
	pub fn rotate(&mut self, axis1 : VecIndex, axis2 : VecIndex, speed_mult : Field) {
		self.frame = self.frame.dot(
			rotation_matrix(
			self.frame[axis1], self.frame[axis2],
			Some(speed_mult*Self::ANG_SPEED)));
		self.update();
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