pub mod convex;
pub mod single_face;

pub mod face;
pub mod buildshapes;

use crate::colors::Color;
use crate::vector;
use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex};
use super::{Line,Plane,line_plane_intersect,Transform,Transformable};
pub use face::Face;
pub use convex::Convex; pub use single_face::SingleFace;

use specs::{Component, VecStorage};
use std::fmt;

#[derive(Component)]
#[storage(VecStorage)]
pub enum ShapeType<V: VectorTrait> {
    Convex(convex::Convex),
    SingleFace(single_face::SingleFace<V>)
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
    pub verts_ref : Vec<V>,
    pub verts : Vec<V>,
    pub edges : Vec<Edge>,
    pub faces : Vec<Face<V>>,
    ref_frame : V::M,
    pub radius : Field,
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
            verts,
            edges,
            faces,
            ref_frame : V::M::id(),
            radius,
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
    pub fn update(&mut self, transformation: &Transform<V>) {
        for (v,vr) in self.verts.iter_mut().zip(self.verts_ref.iter()) {
            *v = transformation.transform_vec(vr);
        }
        for face in &mut self.faces {
            face.normal = *transformation.frame * face.normal_ref;
            face.center = transformation.transform_vec(face.center_ref);
            face.threshold = face.normal.dot(face.center);
        }
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
        face.center_ref = vector::barycenter(&face_verts);
    }
    new_shape.radius = Shape::calc_radius(&new_verts);
    new_shape.verts_ref = new_verts;
    new_shape.update();
    new_shape
}
    pub fn update_visibility(&mut self, camera_pos : V, transparent : bool) {
        for face in self.faces.iter_mut() {
            if transparent {
                face.visible = true;
            }
            else {
                face.update_visibility(camera_pos);
            }
        }
    }
    pub fn with_color(mut self, color : Color) -> Self {
        for face in &mut self.faces {
            face.set_color(color);
        }
        self
    }
    pub fn calc_radius(verts : &Vec<V>) -> Field {
        verts.iter().map(|v| v.norm_sq()).fold(0. / 0., Field::max).sqrt()
    }
}
impl<V: VectorTrait> Transformable<V> for Shape<V> {
    fn set_identity(mut self) -> Self {
        self.update(&Transform::identity());
        self
    }
    fn transform(&mut self, transformation: Transform<V>) {
        self.pos = transformation.pos;
        self.frame = self.frame.dot(transformation.frame);
        self.update();
    }
}

