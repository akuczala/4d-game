pub mod buildshapes;
pub mod convex;
pub mod face;
mod shape_library;
pub mod single_face;

use super::{line_plane_intersect, Line, Plane, Transform, Transformable};
use crate::graphics::colors::Color;
use crate::vector;
use crate::vector::{barycenter, Field, VectorTrait};
pub use convex::Convex;
pub use face::Face;
use serde::{Deserialize, Serialize};
pub use single_face::SingleFace;

use crate::geometry::shape::face::FaceGeometry;
use crate::geometry::transform::Scaling;
use std::fmt::{self, Display};

pub use shape_library::*;
pub trait ShapeTypeTrait<V: VectorTrait> {
    fn line_intersect(
        &self,
        shape: &Shape<V>,
        line: &Line<V>,
        visible_only: bool,
        face_visibility: &[bool],
    ) -> Vec<V>;
}

// TODO: rework how ShapeType, Shape, Convex, and SingleFace work.
// do we really need BOTH a ShapeType + Shape for each entity? Can we combine these into a single ADT?
// is there a more general struct we could use to capture both cases?

//TODO: add a third type / replace singleface with struct representing an adhoc collection of (convex) faces
// this would have, in general, a combination of both subface types. Subfaces connecting faces would be of the convex type,
// and subfaces on the boundary would be of the single face type

// so in general, the maximum data needed for a subface is
// the 1 or 2 faces it belongs to
// the normal
#[derive(Clone, Serialize, Deserialize)]
pub enum ShapeType<V> {
    Convex(convex::Convex),
    SingleFace(single_face::SingleFace<V>),
}
impl<V: VectorTrait> ShapeTypeTrait<V> for ShapeType<V> {
    fn line_intersect(
        &self,
        shape: &Shape<V>,
        line: &Line<V>,
        visible_only: bool,
        face_visibility: &[bool],
    ) -> Vec<V> {
        match self {
            ShapeType::Convex(convex) => {
                convex.line_intersect(shape, line, visible_only, face_visibility)
            }
            ShapeType::SingleFace(single_face) => {
                single_face.line_intersect(shape, line, visible_only, face_visibility)
            }
        }
    }
}

pub type VertIndex = usize;
pub type EdgeIndex = usize;
pub type FaceIndex = usize;

#[derive(Clone, Serialize, Deserialize)]
pub struct Edge(pub VertIndex, pub VertIndex);
impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Edge({},{})", self.0, self.1)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Shape<V> {
    pub verts: Vec<V>,
    pub edges: Vec<Edge>,
    pub faces: Vec<Face<V>>,
    pub shape_type: ShapeType<V>,
}

impl<V: VectorTrait> Shape<V> {
    pub fn new(
        verts: Vec<V>,
        edges: Vec<Edge>,
        mut faces: Vec<Face<V>>,
        shape_type: ShapeType<V>,
    ) -> Self {
        //compute vertex indices for all faces
        //we do this before anything else
        //because it is irritating to do when faces and verts are members of shape
        //(having both shape and face mutable causes issues)
        for face in faces.iter_mut() {
            face.calc_vertis(&edges);
            let face_verts = face.vertis.iter().map(|verti| verts[*verti]).collect();
            face.geometry.center = vector::barycenter(&face_verts);
            //try to do this with iterators
            //face.center_ref = vector::barycenter_iter(&mut face.vertis.iter().map(|verti| verts[*verti]));
        }
        Shape {
            verts,
            edges,
            faces,
            shape_type,
        }
    }

    pub fn new_convex(verts: Vec<V>, edges: Vec<Edge>, faces: Vec<Face<V>>) -> Self {
        let shape_type = ShapeType::Convex(Convex::new(&faces));
        Self::new(verts, edges, faces, shape_type)
    }

    //pub fn get_face_verts(&self, face : Face)
    pub fn get_facei_verts(&self, facei: FaceIndex) -> Vec<V> {
        self.faces[facei]
            .vertis
            .iter()
            .map(|vi| self.verts[*vi])
            .collect()
    }
    pub fn point_signed_distance(&self, point: V) -> Field {
        self.faces
            .iter()
            .map(|f| f.plane().point_signed_distance(point))
            .fold(Field::NEG_INFINITY, |a, b| match a > b {
                true => a,
                false => b,
            })
    }
    //returns distance and normal of closest face
    pub fn point_normal_distance(&self, point: V) -> (V, Field) {
        self.faces
            .iter()
            .map(Face::plane)
            .map(|plane| (plane.normal, plane.point_signed_distance(point)))
            .fold((V::zero(), f32::NEG_INFINITY), |(n1, a), (n2, b)| {
                match a > b {
                    true => (n1, a),
                    false => (n2, b),
                }
            })
    }
    //returns distance and normal of closest face
    pub fn point_facei_distance(&self, point: V) -> (usize, Field) {
        self.faces
            .iter()
            .enumerate()
            .map(|(i, f)| (i, f.plane().point_signed_distance(point)))
            .fold((0, f32::NEG_INFINITY), |(i1, a), (i2, b)| match a > b {
                true => (i1, a),
                false => (i2, b),
            })
    }
    pub fn modify(&mut self, transform: &Transform<V, V::M>) {
        for v in self.verts.iter_mut() {
            *v = transform.transform_vec(v);
        }
        for face in self.faces.iter_mut() {
            face.geometry.plane.normal = (transform.frame * face.normal()).normalize();
            face.geometry.center = transform.transform_vec(&face.center());
            face.geometry.plane.threshold = face.normal().dot(face.center());
        }
    }
    pub fn update_from_ref(&mut self, ref_shape: &Shape<V>, transform: &Transform<V, V::M>) {
        for (v, vr) in self.verts.iter_mut().zip(ref_shape.verts.iter()) {
            *v = transform.transform_vec(vr);
        }
        for (face, ref_face) in self.faces.iter_mut().zip(ref_shape.faces.iter()) {
            // todo: use inverse transform matrix on normals, or
            // https://lxjk.github.io/2017/10/01/Stop-Using-Normal-Matrix.html
            face.geometry.plane.normal = (transform.frame * ref_face.normal()).normalize();
            face.geometry.center = transform.transform_vec(&ref_face.center());
            face.geometry.plane.threshold = face.normal().dot(face.center());
        }
        if let ShapeType::SingleFace(ref mut single_face) = self.shape_type {
            single_face.update(&self.verts, &self.faces)
        }
    }
}
// impl<V: VectorTrait> Transformable<V> for Shape<V> {
//     fn transform(&mut self, transformation: Transform<V>) {
//         self.update( &transformation)
//     }
// }
