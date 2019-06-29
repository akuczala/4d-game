pub mod buildshapes;
pub mod buildfloor;
use std::fmt;
use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex,is_close};
use crate::colors::Color;
use itertools::Itertools;
use crate::vector;
use crate::colors::WHITE;
use std::clone::Clone;
//use std::ops::Index;

#[derive(Clone)]
pub struct Line<V : VectorTrait>(pub V,pub V);
impl<V : VectorTrait> fmt::Display for Line<V> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line({},{})", self.0,self.1)
    }
}
impl<V : VectorTrait> Line<V> {
  pub fn map<F,U>(&self, f : F) -> Line<U>
  where U : VectorTrait,
  F : Fn(V) -> U
  {
    Line(f(self.0),f(self.1))
  }
}
#[derive(Clone)]
pub struct Plane<V : VectorTrait> {
  pub normal : V,
  pub threshold : Field
}
impl<V : VectorTrait> fmt::Display for Plane<V> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "n={},th={}", self.normal,self.threshold)
    }
}

pub fn line_plane_intersect<V>(line : Line<V>, plane : Plane<V>) -> Option<V>
where V : VectorTrait
{
  let p0 = line.0; let p1 = line.1;
  let n = plane.normal; let th = plane.threshold;
  let p0n = p0.dot(n); let p1n = p1.dot(n);
  //line is contained in plane
  if is_close(p0n,0.) && is_close(p1n,0.) {
    return None;
  }
  let t = (p0n - th)/(p0n - p1n);
  //plane does not intersect line segment
  if t < 0. || t > 1. {
    return None;
  }
  Some(V::linterp(p0,p1,t))
}

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

#[derive(Clone)]
pub struct Face<V : VectorTrait> {
  pub normal : V, 
  normal_ref : V,

  pub center : V,
  pub center_ref : V,

  pub threshold : Field,

  pub color : Color,
  pub visible : bool,

  pub edgeis : Vec<EdgeIndex>,
  vertis: Vec<VertIndex>

}

impl<V : VectorTrait> Face<V> {
  pub fn new(edgeis : Vec<EdgeIndex>, normal : V) -> Face<V> {
    let face = Face{
      normal : normal,
      normal_ref : normal.clone(),

      center : V::zero(),
      center_ref : V::zero(),

      threshold : 0.0,

      color: WHITE,
      visible: true,

      edgeis : edgeis,
      vertis : Vec::new() 

    };

    face
  }
  //compute vertex indices from edge indices and a list of edges
  fn calc_vertis(&mut self, edges : &Vec<Edge>) {
    let mut vertis : Vec<VertIndex> = Vec::new();
    for edgei in self.edgeis.iter() {
      let edge = &edges[*edgei];
      vertis.push(edge.0);
      vertis.push(edge.1);
    }
    //this is probably inefficient
    for verti in vertis.iter().unique() {
      self.vertis.push(*verti);
    }
  }
  fn update_visibility(&mut self,camera_pos : V)
  {
    self.visible = self.normal.dot(self.center - camera_pos) < 0.0;
  }

}
impl<V : VectorTrait> fmt::Display for Face<V> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      let mut out = format!("n_ref={}, ",self.normal_ref);
      out.push_str("edgeis=[");
      for ei in self.edgeis.iter() {
        out.push_str(&format!("{},",*ei));
      }
      out.push_str("]");
      write!(f,"{}", out)
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
#[derive(Clone)]
pub struct Shape<V : VectorTrait> {
  verts_ref : Vec<V>,
  pub verts : Vec<V>,
  pub edges : Vec<Edge>,
  pub faces : Vec<Face<V>>,
  pub subfaces : Vec<SubFace>,

  pub boundaries : Vec<Plane<V>>,

  ref_frame : V::M,
  frame : V::M,
  pos : V,
  pub scale : Field,
  pub radius : Field,

  pub transparent : bool
}
impl <V : VectorTrait> Shape<V> {
  pub fn new(verts : Vec<V>, edges: Vec<Edge>, mut faces: Vec<Face<V>>) -> Shape<V> {
    //compute vertex indices for all faces
    //we do this before anything else
    //because it is irritating to do when faces and verts are members of shape
    //(having both shape and face mutable causes issues)
    for face in faces.iter_mut() {
      face.calc_vertis(&edges);
      let face_verts = face.vertis.iter().map(|verti| verts[*verti]).collect();
      face.center_ref = vector::barycenter(face_verts);
      //try to do this with iterators
      //face.center_ref = vector::barycenter_iter(&mut face.vertis.iter().map(|verti| verts[*verti]));
      face.center = face.center_ref.clone();
    }
    let radius = verts.iter().map(|v| v.norm_sq()).fold(0./0., Field::max).sqrt();
    let mut shape = Shape{
    verts_ref : verts.clone(),
    verts : verts,
    edges : edges,
    subfaces : calc_subfaces(&faces), //must come before faces : faces
    faces : faces,
    
    boundaries : Vec::new(),
    ref_frame : V::M::id(),
    frame : V::M::id(),
    pos : V::zero(),
    scale : 1.0,
    radius : radius,
    transparent: false
    };
    shape.update();
    shape
  }

  //pub fn get_face_verts(&self, face : Face)
  pub fn get_facei_verts(&self, facei : FaceIndex) -> Vec<V>
  {
    self.faces[facei].vertis.iter().map(|vi| self.verts[*vi]).collect()
  }
  pub fn transform(&mut self) {
    for (v,vr) in self.verts.iter_mut().zip(self.verts_ref.iter()) {
      *v = self.frame * (*vr * self.scale) + self.pos;
    }
    for face in &mut self.faces {
      face.normal = self.frame * face.normal_ref;
      face.center = self.frame * face.center_ref + self.pos;
      face.threshold = face.normal.dot(face.center);
    }
  }
  pub fn update(&mut self) {
    self.transform();
  }
  pub fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle : Field) {
    let rot_mat = vector::rotation_matrix(self.frame[axis1],self.frame[axis2],Some(angle));
    self.frame = self.frame.dot(rot_mat);
    self.update();
  }
  pub fn set_pos(mut self, pos : &V) -> Self {
    self.pos = *pos;
    self.update();
    self
  }
  pub fn get_pos(& self) -> &V {
    &self.pos
  }
  pub fn stretch(&self, scales : &V) -> Self {
  let mut new_shape = self.clone();
  let new_verts : Vec<V> = self.verts_ref.iter()
    .map(|v| v.zip_map(*scales,|vi,si| vi*si)).collect();
  //need to explicitly update this as it stands
  //need to have a clear differentiation between
  //changes to mesh (verts_ref and center_ref) and
  //changes to position/orientation/scaling of mesh

  for face in &mut new_shape.faces {
        let face_verts = face.vertis.iter().map(|verti| new_verts[*verti]).collect();
    face.center_ref = vector::barycenter(face_verts);
  }
  new_shape.verts_ref = new_verts;
  new_shape.update();
  new_shape
}
  pub fn update_visibility(&mut self, camera_pos : V) {
    for face in self.faces.iter_mut() {
      if self.transparent {
        face.visible = true;
      }
      else {
        face.update_visibility(camera_pos);
      }
    }
  }
  pub fn set_color(mut self, color : Color) -> Self {
    for face in &mut self.faces {
      face.color = color;
    }
    self.update();
    self
  }

}

