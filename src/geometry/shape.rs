pub mod convex;
pub mod single_face;

pub mod face;

pub mod buildshapes;

use std::collections::HashMap;
use crate::graphics::colors::Color;
use crate::vector;
use crate::vector::{VectorTrait,Field};
use super::{Line,Plane,line_plane_intersect,Transform,Transformable};
pub use face::Face;
pub use convex::Convex; pub use single_face::SingleFace;

use specs::{Component, VecStorage};
use std::fmt;
use crate::geometry::transform::Scaling;

#[derive(Component,PartialEq,Eq,Hash,Clone)]
#[storage(VecStorage)]
pub struct ShapeLabel(pub String);

pub struct RefShapes<V: VectorTrait>(HashMap<ShapeLabel,Shape<V>>);
impl<V: VectorTrait> RefShapes<V> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&self, key: &ShapeLabel) -> Option<&Shape<V>> {
        self.0.get(key)
    }
    pub fn insert(&mut self, key: ShapeLabel, value: Shape<V>) -> Option<Shape<V>> {
        self.0.insert(key, value)
    }
    pub fn remove(&mut self, key: &ShapeLabel) -> Option<Shape<V>> {
        self.0.remove(key)
    }
}
impl<V: VectorTrait> Default for RefShapes<V> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

pub trait ShapeTypeTrait<V: VectorTrait> {
    fn line_intersect(&self, shape: &Shape<V>, line : &Line<V>, visible_only : bool) -> Vec<V>;
}
#[derive(Component,Clone)]
#[storage(VecStorage)]
pub enum ShapeType<V: VectorTrait> {
    Convex(convex::Convex),
    SingleFace(single_face::SingleFace<V>)
}
impl<V: VectorTrait> ShapeTypeTrait<V> for ShapeType<V> {
    fn line_intersect(&self, shape: &Shape<V>, line : &Line<V>, visible_only : bool) -> Vec<V> {
        match self {
            ShapeType::Convex(convex) => convex.line_intersect(shape, line, visible_only),
            ShapeType::SingleFace(single_face) => single_face.line_intersect(shape, line, visible_only),
        }
    }
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

#[derive(Clone,Component)]
#[storage(VecStorage)]
pub struct Shape<V : VectorTrait> {
    pub verts : Vec<V>,
    pub edges : Vec<Edge>,
    pub faces : Vec<Face<V>>
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
            face.center = vector::barycenter(&face_verts);
            //try to do this with iterators
            //face.center_ref = vector::barycenter_iter(&mut face.vertis.iter().map(|verti| verts[*verti]));
        }
        let mut shape = Shape{
            verts,
            edges,
            faces,
        };
        //shape.update(&Transform::identity());
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
    pub fn update(&mut self, transform: &Transform<V>) {
        for v in self.verts.iter_mut() {
            *v = transform.transform_vec(v);
        }
        for face in self.faces.iter_mut() {
            face.normal = transform.frame * face.normal;
            face.center = transform.transform_vec(&face.center);
            face.threshold = face.normal.dot(face.center);
        }
    }
    pub fn update_from_ref(&mut self, ref_shape: &Shape<V>, transform: &Transform<V>) {
        for (v,vr) in self.verts.iter_mut().zip(ref_shape.verts.iter()) {
            *v = transform.transform_vec(vr);
        }
        for (face, ref_face) in self.faces.iter_mut().zip(ref_shape.faces.iter()) {
            face.normal = transform.frame * ref_face.normal;
            face.center = transform.transform_vec(&ref_face.center);
            face.threshold = face.normal.dot(face.center);
        }
    }
    // TODO why isn't deprecated in favor of the Transformable trait's method?
    pub fn stretch(&self, ref_shape: &Shape<V>, scales : &V) -> Self {
        let mut new_shape = self.clone();
        let new_verts: Vec<V> = ref_shape.verts.iter()
            .map(|v| v.zip_map(*scales,|vi,si| vi*si)).collect();

        for face in new_shape.faces.iter_mut() {
                    let face_verts = face.vertis.iter().map(|verti| new_verts[*verti]).collect();
            face.center = vector::barycenter(&face_verts);
        }
        new_shape.verts = new_verts;
        new_shape.update(&Transform::identity());
        new_shape
    }
    pub fn update_visibility(&mut self, camera_pos : V, two_sided : bool) {
        for face in self.faces.iter_mut() {
            face.update_visibility(camera_pos, two_sided);
        }
    }
    pub fn with_color(mut self, color : Color) -> Self {
        for face in &mut self.faces {
            face.set_color(color);
        }
        self
    }
}
impl<V: VectorTrait> Transformable<V> for Shape<V> {
    fn transform(&mut self, transformation: Transform<V>) {
        self.update( &transformation)
    }
}

