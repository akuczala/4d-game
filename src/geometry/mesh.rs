
use crate::geometry::face::{FaceBuilder,Face,FaceState};
//use crate::geometry
use crate::geometry::{Plane};
use crate::vector::{VectorTrait,MatrixTrait,Field,Rotatable};

use crate::colors::*;

use std::fmt;
use itertools::Itertools;

pub type VertIndex = usize;
pub type EdgeIndex = usize;
pub type FaceIndex = usize;

#[derive(Clone)]
pub struct Edge(pub VertIndex,pub VertIndex);
impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Edge({},{})", self.0, self.1)
    }
}

//#[derive(Clone)]
pub struct MeshBuilder<V : VectorTrait> {
  pub verts : Vec<V>,
  pub edges : Vec<Edge>,
  pub face_builders : Vec<FaceBuilder<V>>,
}
impl<V : VectorTrait> MeshBuilder<V> {
  pub fn build(&self) -> Mesh<V> {
    let faces : Vec<Face<V>> = self.face_builders
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

    for face in &mut self.face_builders {
      face.normal = frame * face.normal;
    }

  }
}
//would like to be able to manipulate meshes by rotating them
//maybe use some kind of mesh builder struct?
//needs update function
pub struct Mesh<V : VectorTrait> {
  pub verts : Vec<V>,
  pub edges : Vec<Edge>,
  pub faces : Vec<Face<V>>,
  pub subfaces : Vec<SubFace>,

}

impl<V : VectorTrait> Mesh<V> {

  fn calc_radius(verts : &Vec<V>) -> Field {
      verts.iter().map(|v| v.norm_sq()).fold(0./0., Field::max).sqrt()
  }
pub fn stretch(&self, scales : &V) {
  unimplemented![]
  // let mut new_shape = self.clone();
  // let new_verts : Vec<V> = self.verts.iter()
  //  .map(|v| v.zip_map(*scales,|vi,si| vi*si)).collect();
  // //need to explicitly update this as it stands
  // //need to have a clear differentiation between
  // //changes to mesh (verts_ref and center_ref) and
  // //changes to position/orientation/scaling of mesh

  // for face in &mut new_shape.faces {
  //     let face_verts = face.vertis.iter().map(|verti| new_verts[*verti]).collect();
  //  face.center = barycenter(face_verts);
  // }
  // new_shape.verts = new_verts;
  // new_shape.update();

  }
}
impl<V : VectorTrait> Rotatable<V> for Mesh<V> {
  //always identity; we just rotate from the current verts
  fn get_frame(&self) -> V::M {
    V::M::id()
  }
  fn set_frame(&mut self, frame : V::M) {
    self.verts = self.verts.iter().map(|&v| frame*v).collect();
    for face in &mut self.faces {
      face.transform(&frame,&V::zero());
    }
  }
}
//every MeshState should be the slave of an object
pub struct MeshState<'a,V : VectorTrait> {
  pub ref_mesh : &'a Mesh<V>,
  pub pos : V, //would ideally be ref but i want to avoid spiderman pointing
  pub verts : Vec<V>,
  pub faces : Vec<FaceState<'a,V>>,
  pub ref_frame : V::M, // could probably replace this by the identity matrix
  pub boundaries : Vec<Plane<V>>,
  pub transparent : bool,
  pub scale : Field,
  pub radius : Field,
}
impl<'a,V : VectorTrait> MeshState<'a,V> {
  pub fn new(mesh : &'a Mesh<V>,&pos : &V) -> Self {
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
  pub fn transform(&mut self, &frame : &V::M, &pos : &V) {
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

#[derive(Clone)]
pub struct SubFace {
  pub faceis : (FaceIndex,FaceIndex)
}
impl fmt::Display for SubFace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SubFace({},{})", self.faceis.0, self.faceis.1)
    }
}
//find indices of (d-1) faces that are joined by a (d-2) edge
fn calc_subfaces<V : VectorTrait>(faces : &Vec<Face<V>>) -> Vec<SubFace> {
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
fn count_common_edges<V : VectorTrait>(face1 : &Face<V>, face2 : &Face<V>) -> usize {
  let total_edges = face1.edgeis.len() + face2.edgeis.len();
  let unique_edges = face1.edgeis.iter()
    .chain(face2.edgeis.iter())
    .unique()
    .count();
  total_edges - unique_edges

}