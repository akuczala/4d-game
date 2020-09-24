use crate::colors::Color;
use crate::vector;
use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex};
use crate::geometry::{Line,Edge,Face,Plane,FaceIndex,line_plane_intersect,ShapeTrait};

use specs::{Component, VecStorage};
use itertools::Itertools;

#[derive(Clone,Component)]
#[storage(VecStorage)]
pub struct Shape<V : VectorTrait> {
    pub verts_ref : Vec<V>,
    pub verts : Vec<V>,
    pub edges : Vec<Edge>,
    pub faces : Vec<Face<V>>,
    pub subfaces : Vec<SubFace>,

    //pub boundaries : Vec<Plane<V>>,

    ref_frame : V::M,
    frame : V::M,
    pos : V,
    pub scale : Field,
    pub radius : Field,

    //pub transparent : bool
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
            face.center_ref = vector::barycenter(&face_verts);
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
        
        //boundaries : Vec::new(),
        ref_frame : V::M::id(),
        frame : V::M::id(),
        pos : V::zero(),
        scale : 1.0,
        radius : radius,
        //transparent: false
        };
        shape.update();
        shape
    }

    //pub fn get_face_verts(&self, face : Face)
    pub fn get_facei_verts(&self, facei : FaceIndex) -> Vec<V>
    {
        self.faces[facei].vertis.iter().map(|vi| self.verts[*vi]).collect()
    }
    pub fn point_signed_distance(&self, point : V) -> Field {
        self.faces.iter().map(|f| f.normal.dot(point) - f.threshold).fold(Field::NEG_INFINITY,|a,b| match a > b {true => a, false => b})
    }
    //returns distance and normal of closest face
    pub fn point_normal_distance(&self, point : V) -> (V, Field) {
         self.faces.iter().map(|f| (f.normal, f.normal.dot(point) - f.threshold))
            .fold((V::zero(),f32::NEG_INFINITY),|(n1,a),(n2,b)| match a > b {true => (n1,a), false => (n2,b)})
    }
    //returns distance and normal of closest face
    pub fn point_facei_distance(&self, point : V) -> (usize, Field) {
         self.faces.iter().enumerate().map(|(i,f)| (i, f.normal.dot(point) - f.threshold))
            .fold((0,f32::NEG_INFINITY),|(i1,a),(i2,b)| match a > b {true => (i1,a), false => (i2,b)})
    }
    pub fn point_within(&self, point : V, distance : Field) -> bool {
        self.faces.iter().all(|f| f.normal.dot(point) - f.threshold < distance)
    }
    //returns points of intersection with shape
    pub fn line_intersect(&self, line : &Line<V>, visible_only : bool) -> Vec<V> {//impl std::iter::Iterator<Item=Option<V>> {
        let mut out_points = Vec::<V>::new();
        for face in self.faces.iter().filter(|f| !visible_only || f.visible) {
            if let Some(p) = line_plane_intersect(line,&Plane{normal : face.normal, threshold : face.threshold}) {
                if crate::vector::is_close(self.point_signed_distance(p),0.) {
                    out_points.push(p);
                }
            }
        }
     out_points
    }

    pub fn calc_boundary(face1 : &Face<V>, face2 : &Face<V>, origin : V) -> Plane<V>
    {
        let (n1,n2) = (face1.normal,face2.normal);
        let (th1,th2) = (face1.threshold, face2.threshold);

        //k1 and k2 must have opposite signs
        let k1 = n1.dot(origin) - th1;
        let k2 = n2.dot(origin) - th2;
        //assert!(k1*k2 < 0.0,"k1 = {}, k2 = {}",k1,k2);

        let t = k1/(k1 - k2);

        let n3 = V::linterp(n1, n2, t);
        let th3 = crate::vector::scalar_linterp(th1, th2, t);

        Plane{normal : n3, threshold: th3}
    }
}
impl<V : VectorTrait> ShapeTrait<V> for Shape<V> {
    fn transform(&mut self) {
        for (v,vr) in self.verts.iter_mut().zip(self.verts_ref.iter()) {
            *v = self.frame * (*vr * self.scale) + self.pos;
        }
        for face in &mut self.faces {
            face.normal = self.frame * face.normal_ref;
            face.center = self.frame * face.center_ref + self.pos;
            face.threshold = face.normal.dot(face.center);
        }
    }
    fn update(&mut self) {
        self.transform();
    }
    fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle : Field) {
        let rot_mat = vector::rotation_matrix(self.frame[axis1],self.frame[axis2],Some(angle));
        self.frame = self.frame.dot(rot_mat);
        self.update();
    }
    fn set_pos(mut self, pos : &V) -> Self {
        self.pos = *pos;
        self.update();
        self
    }
    fn get_pos(& self) -> &V {
        &self.pos
    }
    fn stretch(&self, scales : &V) -> Self {
    let mut new_shape = self.clone();
    let new_verts : Vec<V> = self.verts_ref.iter()
        .map(|v| v.zip_map(*scales,|vi,si| vi*si)).collect();
    //need to explicitly update this as it stands
    //need to have a clear differentiation between
    //changes to mesh (verts_ref and center_ref) and
    //changes to position/orientation/scaling of mesh

    for face in &mut new_shape.faces {
                let face_verts = face.vertis.iter().map(|verti| new_verts[*verti]).collect();
        face.center_ref = vector::barycenter(&face_verts);
    }
    new_shape.radius = Shape::calc_radius(&new_verts);
    new_shape.verts_ref = new_verts;
    new_shape.update();
    new_shape
}
    fn update_visibility(&mut self, camera_pos : V, transparent : bool) {
        for face in self.faces.iter_mut() {
            if transparent {
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
    fn calc_radius(verts : &Vec<V>) -> Field {
        verts.iter().map(|v| v.norm_sq()).fold(0./0., Field::max).sqrt()
    }
    fn calc_boundaries(&self, origin : V) -> Vec<Plane<V>> {

        let faces = &self.faces; let subfaces = &self.subfaces;

        let mut boundaries : Vec<Plane<V>> = Vec::new();

        for subface in subfaces {
            let face1 = &faces[subface.faceis.0];
            let face2 = &faces[subface.faceis.1];
            if face1.visible == !face2.visible {
                let boundary = Self::calc_boundary(face1, face2, origin);
                boundaries.push(boundary);
            }
        }
        //visible faces are boundaries
        for face in faces {
            if face.visible {
                boundaries.push(Plane{
                    normal : face.normal, threshold : face.threshold
                })
            }
        }
        boundaries
    }
}


#[derive(Clone)]
pub struct SubFace {
  pub faceis : (FaceIndex,FaceIndex)
}
use std::fmt;
impl fmt::Display for SubFace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SubFace({},{})", self.faceis.0, self.faceis.1)
    }
}

//find indices of (d-1) faces that are joined by a (d-2) edge
fn calc_subfaces<V : VectorTrait>(faces : &Vec<Face<V>>) -> Vec<SubFace> {
  let mut subfaces : Vec<SubFace> = Vec::new();
  if V::DIM == 2{
    for i in 0..faces.len() {
      for j in 0..i {
        if count_common_verts(&faces[i],&faces[j]) >= 1 {
          subfaces.push(SubFace{faceis : (i,j)})
        }
      }
    }
    return subfaces
  }
  let n_target = match V::DIM {
    3 => 1,
    4 => 2,
    _ => panic!("Invalid dimension for computing subfaces")
  };
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
fn count_common_verts<V : VectorTrait>(face1 : &Face<V>, face2 : &Face<V>) -> usize {
  let total_verts = face1.vertis.len() + face2.vertis.len();
  let unique_verts = face1.vertis.iter()
    .chain(face2.vertis.iter())
    .unique()
    .count();
  total_verts - unique_verts

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

#[test]
fn test_linspace() {
    use crate::vector::{is_close,linspace};
    assert!(linspace(-2.5,2.5,9).zip(vec![-2.5  , -1.875, -1.25 , -0.625,  0.   ,  0.625,  1.25 ,  1.875,
                2.5  ]).all(|(a,b)| is_close(a,b)))
}

//prints points at different distances from prism
#[test]
fn test_point_within2() {
    use colored::*;
    use vector::{Vec3,linspace};
    let shape = crate::geometry::buildshapes::build_prism_3d(1.0, 1.0, 4);
    for x in linspace(-2.,2.,40) {
        let mut line = "".to_string();
        for y in linspace(-2.,2.,40) {
            let point = Vec3::new(x,y,0.);
            // let newstr = match shape.point_within(Vec3::new(x,y,0.),0.) {
            //   true => "+", false => "_"
            // };
            let (i,dist) = shape.point_facei_distance(point);
            //println!("{}",dist);
            //let newstr = match dist {a if a > 1. => "#", a if a > 0. => "+", a if a <= 0. => "_", _ => "^"};
            let mut newstr = match i {1 => "1".blue(), 2 => "2".yellow(), 3 => "3".cyan(), 4 => "4".green(), _ => "_".red()};
            if dist > 1. {
                newstr = "+".to_string().white();
            }
            line = format!("{} {}",line,newstr);
            
        }
        println!("{}",line);
    }
    assert!(false); //forces cargo test to print this
    //assert!(!shape.point_within(point,0.))
}