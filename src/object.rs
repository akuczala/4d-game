
use crate::geometry::{Edge,Face,SubFace,Plane,EdgeIndex,VertIndex};
use crate::draw::{Texture,TextureMapping};
use crate::vector::{VectorTrait,MatrixTrait,Field,barycenter,Rotatable};
use crate::colors::*;

use std::fmt;
use itertools::Itertools;

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

fn calc_subfaces<V : VectorTrait>(faces : &Vec<FaceNew<V>>) -> Vec<SubFace> {
  let n_target = match V::DIM {
    3 => 1,
    4 => 2,
    _ => panic!("Invalid dimension for computing subfaces")
  };
  let mut subfaces : Vec<SubFace> = Vec::new();
  for i in 0..faces.len() {
    for j in 0..i {
      if count_common_edges(&faces[i],&faces[j]) >= n_target {
        subfaces.push(SubFace{faceis : (i,j)})
      }
    }
  }
  subfaces
}
fn count_common_edges<V : VectorTrait>(face1 : &FaceNew<V>, face2 : &FaceNew<V>) -> usize {
  let total_edges = face1.edgeis.len() + face2.edgeis.len();
  let unique_edges = face1.edgeis.iter()
    .chain(face2.edgeis.iter())
    .unique()
    .count();
  total_edges - unique_edges
}

// pub enum MeshBuilder<V : VectorTrait> {
// 	Init,
// 	HasMesh{mesh : Mesh<V>}
// }
//needs update function
pub struct MeshBuilder<V : VectorTrait> {
	verts : Vec<V>,
	edges : Vec<Edge>,
	face_builders : Vec<FaceBuilder<V>>,
}
impl<V : VectorTrait> MeshBuilder<V> {
	pub fn build(&self) -> Mesh<V> {
		let faces : Vec<FaceNew<V>> = self.face_builders
			.iter()
			.map(|v| v.build(&self))
			.collect();
		//let radius = Mesh::calc_radius(&self.verts);
		Mesh{
			verts : self.verts.clone(),
			edges : self.edges.clone(),
			subfaces : calc_subfaces(&faces), //must come before faces : faces
			faces : faces
		}
	}
}
impl<V : VectorTrait> Rotatable<V> for MeshBuilder<V> {
	//always identity; we just rotate from the current verts
	fn get_frame(&self) -> V::M {
		V::M::id()
	}
	fn set_frame(&mut self, frame : V::M) {
		self.verts = self.verts.iter().map(|&v| frame*v).collect();
		//need to update face_builders here
	}
}
//would like to be able to manipulate meshes by rotating them
//maybe use some kind of mesh builder struct?
//needs update function
pub struct Mesh<V : VectorTrait> {
	verts : Vec<V>,
	edges : Vec<Edge>,
	faces : Vec<FaceNew<V>>,
	subfaces : Vec<SubFace>,

}

impl<V : VectorTrait> Mesh<V> {

	fn calc_radius(verts : &Vec<V>) -> Field {
    	verts.iter().map(|v| v.norm_sq()).fold(0./0., Field::max).sqrt()
	}
pub fn stretch(&self, scales : &V) {
	let mut new_shape = self.clone();
	let new_verts : Vec<V> = self.verts.iter()
		.map(|v| v.zip_map(*scales,|vi,si| vi*si)).collect();
	//need to explicitly update this as it stands
	//need to have a clear differentiation between
	//changes to mesh (verts_ref and center_ref) and
	//changes to position/orientation/scaling of mesh

	for face in &mut new_shape.faces {
	    let face_verts = face.vertis.iter().map(|verti| new_verts[*verti]).collect();
		face.center = barycenter(face_verts);
	}
	new_shape.verts = new_verts;
	new_shape.update();

	}
}
impl<V : VectorTrait> Rotatable<V> for Mesh<V> {
	//always identity; we just rotate from the current verts
	fn get_frame(&self) -> V::M {
		V::M::id()
	}
	fn set_frame(&mut self, frame : V::M) {
		self.verts = self.verts.iter().map(|&v| frame*v).collect();
	}
}
//every MeshState should be the slave of an object
pub struct MeshState<'a,V : VectorTrait> {
	ref_mesh : &'a Mesh<V>,
	pos : V, //would ideally be ref but i want to avoid spiderman pointing
	verts : Vec<V>,
	faces : Vec<FaceState<'a,V>>,
	ref_frame : V::M, // could probably replace this by the identity matrix
	boundaries : Vec<Plane<V>>,
	transparent : bool,
	pub scale : Field,
	pub radius : Field,
}
impl<'a,V : VectorTrait> MeshState<'a,V> {
	fn new(mesh : &'a Mesh<V>,&pos : &V) -> Self {
		MeshState{
			ref_mesh : &mesh,
			pos : pos,
			verts : mesh.verts.clone(),
			faces : mesh.faces
				.iter()
				.map(|f| FaceState::new(f))
				.collect(),
			ref_frame : V::M::id(),
			boundaries : Vec::new(),
			transparent : false,
			scale : 1.0,
			radius : Mesh::calc_radius(&mesh.verts)
		}
	}
}

impl<'a,V : VectorTrait> MeshState<'a,V> {
	fn transform(&mut self, &frame : &V::M, &pos : &V) {
		self.pos = pos; //tracks object's pos
	    for (v,vr) in self.verts.iter_mut().zip(self.ref_mesh.verts.iter()) {
	      *v = frame * (*vr * self.scale) + pos;
	    }
		for face in &mut self.faces {
			face.transform(&frame, &pos);
		}
	}
	fn update_visibility(&mut self, camera_pos : V) {
    for face in self.faces.iter_mut() {
      if self.transparent {
        face.visible = true;
      }
      else {
        face.update_visibility(camera_pos);
      }
    }
  }
  fn set_color(mut self, color : Color) -> Self {
    for face in &mut self.faces {
      face.set_color(color);
    }
    self
  }
}

pub struct FaceState<'a,V : VectorTrait> {
	pub ref_face : &'a FaceNew<V>,

	pub normal : V,
	pub center : V,

	pub threshold : Field,

	pub texture : Texture<V::SubV>,
	pub texture_mapping : TextureMapping,
	pub visible : bool,

}
impl<'a,V : VectorTrait> FaceState<'a,V>{
	fn new(ref_face : &'a FaceNew<V>) -> Self {
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
	fn transform(&mut self, &frame : &V::M, &pos : &V) {
		self.normal = frame * self.normal;
		self.center = frame * self.center + pos;
		self.threshold = self.normal.dot(self.center);
	}
	fn update_visibility(&mut self,camera_pos : V)
	{
		self.visible = self.normal.dot(self.center - camera_pos) < 0.0;
	}
	fn set_color(&mut self, color : Color) {
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
	pub fn build(&self, mesh_builder : &MeshBuilder<V>) -> FaceNew<V> {
		let vertis = Self::calc_vertis(&self.edgeis, &mesh_builder.edges);
	    let mut face = FaceNew{
	    	edgeis : self.edgeis.clone(),
	    	vertis : vertis,
			normal : self.normal.clone(),
			center : V::zero(),
	    };
	    let verts = FaceNew::get_verts(&face,&mesh_builder.verts);
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
pub struct FaceNew<V : VectorTrait> {
	pub edgeis : Vec<EdgeIndex>,
	pub vertis: Vec<VertIndex>,

	pub normal : V, 
	pub center : V,

}

impl<V : VectorTrait> FaceNew<V> {
  // pub fn new(edgeis : Vec<EdgeIndex>, normal : V, mesh : &Mesh<V>) -> FaceNew<V> {
  	
  // }
  fn get_verts(face : &FaceNew<V>, mesh_verts : &Vec<V>) -> Vec<V> {
  	face.vertis.iter().map(|verti| mesh_verts[*verti]).collect()
  }

}
impl<V : VectorTrait> fmt::Display for FaceNew<V> {
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