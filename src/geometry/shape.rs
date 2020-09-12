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

}