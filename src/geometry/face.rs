
use crate::draw::{Texture,TextureMapping};
use crate::geometry::mesh::{VertIndex,EdgeIndex,MeshBuilder,Edge};
use crate::vector::{VectorTrait,MatrixTrait,Field,barycenter,Rotatable};
use crate::colors::*;

use std::fmt;
use itertools::Itertools;

pub struct FaceState<'a,V : VectorTrait> {
	pub ref_face : &'a Face<V>,

	pub normal : V,
	pub center : V,

	pub threshold : Field,

	pub texture : Texture<V::SubV>,
	pub texture_mapping : TextureMapping,
	pub visible : bool,

}
impl<'a,V : VectorTrait> FaceState<'a,V>{
	pub fn new(ref_face : &'a Face<V>) -> Self {
		FaceState{
			normal : ref_face.normal,
			center : ref_face.center,
			threshold : 0.0,
			texture : Texture::DefaultLines{color : WHITE},
      		texture_mapping : TextureMapping{frame_vertis : Vec::new(), origin_verti : 0}, //this is pretty sloppy
      		visible: true,
      		ref_face : ref_face,	
		}
	}
	pub fn transform(&mut self, &frame : &V::M, &pos : &V) {
		self.normal = frame * self.normal;
		self.center = frame * self.center + pos;
		self.threshold = self.normal.dot(self.center);
	}
	pub fn update_visibility(&mut self,camera_pos : V)
	{
		self.visible = self.normal.dot(self.center - camera_pos) < 0.0;
	}
	pub fn set_color(&mut self, color : Color) {
		take_mut::take(&mut self.texture,|tex| tex.set_color(color));
	}
}
//feed to the mesh
pub struct FaceBuilder<V : VectorTrait> {
	pub edgeis : Vec<EdgeIndex>,
	pub normal : V,  //consider storing the normal parity rather than vector
}
//could compute normal parity in a new(edgeis,normal) function
impl< V: VectorTrait> FaceBuilder<V> {
	pub fn build(&self, mesh_builder : &MeshBuilder<V>) -> Face<V> {
		let vertis = Self::calc_vertis(&self.edgeis, &mesh_builder.edges);
	    let mut face = Face{
	    	edgeis : self.edgeis.clone(),
	    	vertis : vertis,
			normal : self.normal.clone(),
			center : V::zero(),
	    };
	    let verts = Face::get_verts(&face,&mesh_builder.verts);
	    face.center = barycenter(verts);
	    face
	}
	//compute vertex indices from edge indices and a list of edges
	fn calc_vertis(edgeis : &Vec<EdgeIndex>, edges : &Vec<Edge>) -> Vec<VertIndex> {
		let mut vertis : Vec<VertIndex> = Vec::new();
		for edgei in edgeis.iter() {
		  let edge = &edges[*edgei];
		  vertis.push(edge.0);
		  vertis.push(edge.1);
		}
		vertis.into_iter().unique().collect()
  }
}
pub struct Face<V : VectorTrait> {
	pub edgeis : Vec<EdgeIndex>,
	pub vertis: Vec<VertIndex>,

	pub normal : V, 
	pub center : V,

}

impl<V : VectorTrait> Face<V> {
  // pub fn new(edgeis : Vec<EdgeIndex>, normal : V, mesh : &Mesh<V>) -> Face<V> {
  	
  // }
  pub fn get_verts(face : &Face<V>, mesh_verts : &Vec<V>) -> Vec<V> {
  	face.vertis.iter().map(|verti| mesh_verts[*verti]).collect()
  }
  pub fn transform(&mut self, &frame : &V::M, &pos : &V) {
		self.normal = frame * self.normal;
		self.center = frame * self.center + pos;
	}

}

impl<V : VectorTrait> fmt::Display for Face<V> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      let mut out = format!("n_ref={}, ",self.normal);
      out.push_str("edgeis=[");
      for ei in self.edgeis.iter() {
        out.push_str(&format!("{},",*ei));
      }
      out.push_str("]");
      write!(f,"{}", out)
    }
}