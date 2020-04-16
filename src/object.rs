use crate::geometry::mesh::{Mesh,MeshState};
use crate::vector::{VectorTrait,MatrixTrait,Rotatable,Translatable};

pub struct Object<'a,V : VectorTrait> {

	pub maybe_mesh_state : Option<MeshState<'a,V>>,
	pub frame : V::M,
	pub pos : V,
}
impl<'a,V : VectorTrait> Object<'a,V>{
	pub fn new() -> Self {
		Object{
			maybe_mesh_state : None,
			frame : V::M::id(),
			pos : V::zero(),
		}
	}
	pub fn new_mesh(&mut self, mesh : &'a Mesh<V>) {
		self.maybe_mesh_state = Some(MeshState::new(mesh,&self.pos));
	}
}

impl<'a,V : VectorTrait> Rotatable<V> for Object<'a,V> {
	fn get_frame(&self) -> V::M {
		self.frame
	}
	fn set_frame(&mut self, new_frame : V::M){
		self.frame = new_frame;
		self.update();
	}
}
impl <'a,V:VectorTrait> Object<'a,V> {
	fn update(&mut self) {
		if let Some(mesh_state) = &mut self.maybe_mesh_state {
			mesh_state.transform(&self.frame,&self.pos);
		}
  	}
}

// pub enum MeshBuilder<V : VectorTrait> {
// 	Init,
// 	HasMesh{mesh : Mesh<V>}
// }
//needs update function


