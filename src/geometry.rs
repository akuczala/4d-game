pub mod buildshapes;
pub mod buildfloor;
pub mod shape;

pub use shape::Shape;

use std::fmt;
use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex,is_close};
use crate::colors::Color;
use itertools::Itertools;

use crate::colors::WHITE;
use std::clone::Clone;
use crate::draw::{Texture,TextureMapping};
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

pub fn point_plane_normal_axis<V : VectorTrait>(point : &V, plane : &Plane<V>) -> Field {
  return plane.threshold - point.dot(plane.normal)
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
pub struct Sphere<V : VectorTrait>{pos : V, radius : Field}


//returns either none or pair of intersecting points
//note that tm and p are NOT bound between 0 and 1
pub fn sphere_line_intersect<V : VectorTrait>(line : Line<V>, r : Field) -> Option<Line<V>> {

    let v0 = line.0;
    let v1 = line.1;
    let dv = v1 - v0;
    let dv_norm = dv.norm();
    let dv = dv / dv_norm;

    //in our case, sphere center is the origin
    let v0_rel = v0;  // - sphere_center
    let v0r_dv = v0_rel.dot(dv);

    let discr = (v0r_dv)*(v0r_dv) - v0_rel.dot(v0_rel) + r * r;

    //print('discr',discr)
    //no intersection with line
    if discr < 0. {
      return None;
    }
        

    let sqrt_discr = discr.sqrt();
    let tm = -v0r_dv - sqrt_discr;
    let tp = -v0r_dv + sqrt_discr;

    //print('tm,tp',tm,tp)
    //no intersection with line segment
    if tm > dv_norm && tp > dv_norm {
      return None;
    }
    if tm < 0. && tp < 0. {
      return None;
    }
    let intersect_points = Line(v0 + dv*tm, v0 + dv*tp);
    
    Some(intersect_points)
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
  pub normal_ref : V,

  pub center : V,
  pub center_ref : V,

  pub threshold : Field,

  pub texture : Texture<V::SubV>,
  pub texture_mapping : TextureMapping,
  pub visible : bool,

  pub edgeis : Vec<EdgeIndex>,
  pub vertis: Vec<VertIndex>

}

impl<V : VectorTrait> Face<V> {
  pub fn new(edgeis : Vec<EdgeIndex>, normal : V) -> Face<V> {
    let face = Face{
      normal : normal.normalize(), //let's make 100% these are normalized
      normal_ref : normal.normalize(),

      center : V::zero(),
      center_ref : V::zero(),

      threshold : 0.0,

      //change texture to reference
      texture : Texture::DefaultLines{color : WHITE},
      texture_mapping : TextureMapping{frame_vertis : Vec::new(), origin_verti : 0}, //this is pretty sloppy
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
  fn set_color(&mut self, color : Color) {
    take_mut::take(&mut self.texture,|tex| tex.set_color(color));
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



