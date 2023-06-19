use crate::constants::PI;
use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex,rotation_matrix};
use crate::geometry::{Plane};
use crate::components::{Transform,Transformable};

pub struct Camera<V, M>
{
	pub heading : M,
	pub plane : Plane<V>,

}
impl<V: VectorTrait> Camera<V, V::M> {
	const SPEED : Field = 1.5;
	const ANG_SPEED : Field = 1.5*PI/3.0;
	const MAX_TILT : Field = 0.99;

	pub fn new(transform : &Transform<V, V::M>) -> Camera<V, V::M> {
		Camera{
			heading : V::M::id(),
			plane : Plane{normal : V::one_hot(-1), threshold : V::one_hot(-1).dot(transform.pos)},

		}
	}
	pub fn look_at(&mut self, transform: &mut Transform<V, V::M>, point : &V) {
		transform.frame = rotation_matrix(*point - transform.pos, V::one_hot(-1), None);
		self.update(&transform);
	}
	pub fn slide(&mut self,  transform: &mut Transform<V, V::M>, direction : V, time : Field) {
		*transform = transform.with_translation(self.get_slide_dpos(direction, time));
		self.update(&transform);
	}
	pub fn get_slide_dpos(&self, direction : V, time : Field) -> V {
		direction.normalize()*Self::SPEED*time
	}
    // TODO do spin and turn differ at all:?
	pub fn spin(&mut self, transform: &mut Transform<V, V::M>, axis1 : VecIndex, axis2 : VecIndex, speed_mult : Field) {
		let rot = rotation_matrix(transform.frame[axis1],transform.frame[axis2],Some(speed_mult*Self::ANG_SPEED));
		transform.frame = transform.frame.dot(rot);
		self.heading = self.heading.dot(rot);
		self.update(&transform);

	}
	//heading-based rotation affecting both frame and heading
	pub fn turn(&mut self, transform: &mut Transform<V, V::M>, axis1 : VecIndex, axis2 : VecIndex, speed_mult : Field) {
		let rot = rotation_matrix(self.heading[axis1],self.heading[axis2],Some(speed_mult*Self::ANG_SPEED));
		transform.frame = transform.frame.dot(rot);
		self.heading = self.heading.dot(rot);
		self.update(&transform);
	}
	//heading-based rotation affecting only frame
	pub fn tilt(&mut self, transform: &mut Transform<V, V::M>, axis1 : VecIndex, axis2 : VecIndex, speed_mult : Field) {
		
		let dot = self.heading[axis1].dot(transform.frame[axis2]); // get projection of frame axis along heading axis

		if dot*speed_mult < 0. ||  dot.abs() < Self::MAX_TILT { //rotate if tilting direction is opposite projection or if < max tilt
		//if true {
			let rot = rotation_matrix(transform.frame[axis1],transform.frame[axis2],Some(speed_mult*Self::ANG_SPEED));
			transform.frame = transform.frame.dot(rot);

			self.update(&transform);
		}
		
	}
	pub fn update_plane(&mut self, transform: &Transform<V, V::M>) {
		self.plane = Plane{
			normal : transform.frame[-1],
			threshold : transform.frame[-1].dot(transform.pos)
		}
	}
	// pub fn update_heading(&mut self) {
	// 	self.heading = self.frame;
	// }
	pub fn update(&mut self, transform: &Transform<V, V::M>) {
		//self.update_heading();
		self.update_plane(&transform);
	}
}

