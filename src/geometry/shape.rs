use crate::colors::Color;
use crate::vector;
use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex};
use crate::geometry::{Edge,Face,SubFace,Plane,FaceIndex,calc_subfaces};

use specs::{Component,System,VecStorage, DenseVecStorage};

#[derive(Clone)]
pub struct Shape<V : VectorTrait> {
  pub verts_ref : Vec<V>,
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

impl<V: VectorTrait> Component for Shape<V> {
    type Storage = DenseVecStorage<Self>;
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
    let radius = Shape::calc_radius(&verts);
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
  new_shape.radius = Shape::calc_radius(&new_verts);
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
      face.set_color(color);
    }
    self
  }
  pub fn calc_radius(verts : &Vec<V>) -> Field {
    verts.iter().map(|v| v.norm_sq()).fold(0./0., Field::max).sqrt()
  }
  pub fn point_signed_distance(&self, point : V) -> Field {
    self.faces.iter().map(|f| f.normal.dot(point) - f.threshold).fold(f32::NEG_INFINITY,|a,b| match a > b {true => a, false => b})
  }
  pub fn point_within(&self, point : V, distance : Field) -> bool {
    self.faces.iter().all(|f| f.normal.dot(point) - f.threshold < distance)
  }
}


#[test]
fn test_point_within() {
  use vector::Vec3;
  let point = Vec3::new(1.2,1.2,1.2);
  let shape = crate::geometry::buildshapes::build_prism_3d(1.0, 1.0, 5);
  for v in shape.faces.iter().map(|f| f.normal.dot(point) - f.threshold) {
    println!("{}",v);
  }
  assert!(!shape.point_within(point,0.))
}

fn linspace(min : Field, max : Field, n : usize) -> impl Iterator<Item=Field> {
  (0..n).map(move |i| (i as Field)/((n-1) as Field)).map(move |f| (1.-f)*min + f*max)
}
#[test]
fn test_linspace() {
  use crate::vector::is_close;
  assert!(linspace(-2.5,2.5,9).zip(vec![-2.5  , -1.875, -1.25 , -0.625,  0.   ,  0.625,  1.25 ,  1.875,
        2.5  ]).all(|(a,b)| is_close(a,b)))
}

//prints points at different distances from prism
#[test]
fn test_point_within2() {
  use vector::Vec3;
  let shape = crate::geometry::buildshapes::build_prism_3d(1.0, 1.0, 5);
  for x in linspace(-2.,2.,40) {
    let mut line = "".to_string();
    for y in linspace(-2.,2.,40) {
      let point = Vec3::new(x,y,0.);
      // let newstr = match shape.point_within(Vec3::new(x,y,0.),0.) {
      //   true => "+", false => "_"
      // };
      let dist = shape.point_signed_distance(point);
      //println!("{}",dist);
      let newstr = match dist {a if a > 1. => "#", a if a > 0. => "+", a if a <= 0. => "_", _ => "^"};
      line = format!("{} {}",line,newstr);
      
    }
    println!("{}",line);
  }
  //assert!(false); //forces cargo test to print this
  //assert!(!shape.point_within(point,0.))
}