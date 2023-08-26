pub mod buildshapes;
pub mod convex;
pub mod face;
pub mod generic;
mod shape_library;
pub mod single_face;
pub mod subface;

use self::generic::GenericShapeType;
use self::subface::SubFace;

use super::{Line, Transform};

use crate::graphics::VertexTrait;
use crate::utils::BranchIterator;

use crate::vector::{Field, VectorTrait};
pub use convex::Convex;
pub use face::Face;
use serde::{Deserialize, Serialize};
pub use single_face::SingleFace;

use std::fmt::{self};

pub use shape_library::*;

#[derive(Clone, Serialize, Deserialize)]
pub enum ShapeType<V> {
    Convex(convex::Convex),
    SingleFace(single_face::SingleFace<V>),
    Generic(GenericShapeType<V>),
}
impl<V: Copy> ShapeType<V> {
    // TODO: find a way to return refs here or don't use
    pub fn get_subfaces(&self) -> Vec<SubFace<V>> {
        match self {
            ShapeType::Convex(Convex { subfaces }) => {
                subfaces.iter().cloned().map(SubFace::Interior).collect()
            }
            ShapeType::SingleFace(SingleFace { subfaces }) => {
                subfaces.iter().cloned().map(SubFace::Boundary).collect()
            }
            ShapeType::Generic(GenericShapeType { subfaces, .. }) => subfaces.clone(),
        }
    }
    #[allow(dead_code)]
    pub fn get_face_subfaces(&self, face_index: FaceIndex) -> impl Iterator<Item = SubFace<V>> {
        self.get_subfaces()
            .into_iter()
            .filter(move |sf| sf.is_face_subface(face_index))
    }
    pub fn to_generic(&self) -> Self {
        let subfaces = self.get_subfaces();
        Self::Generic(GenericShapeType::new(&subfaces))
    }
}
impl<V: VectorTrait> ShapeType<V> {
    pub fn update(&mut self, shape_verts: &[V], shape_faces: &[Face<V>]) {
        match self {
            ShapeType::Convex(_) => (),
            ShapeType::SingleFace(ref mut single_face) => {
                single_face.update(shape_verts, shape_faces)
            }
            ShapeType::Generic(ref mut generic) => {
                for subface in &mut generic.subfaces {
                    match subface {
                        SubFace::Interior(_) => (),
                        SubFace::Boundary(ref mut bsf) => {
                            bsf.update(shape_verts, shape_faces[bsf.facei].normal())
                        }
                    }
                }
            }
        }
    }
}

impl<V: VectorTrait> ShapeType<V> {
    pub fn line_intersect<'a>(
        &'a self,
        shape: &'a Shape<V>,
        line: &'a Line<V>,
        visible_only: bool,
        face_visibility: &'a [bool],
    ) -> impl Iterator<Item = V> + 'a {
        match self {
            ShapeType::Convex(_) => BranchIterator::Option1(Convex::line_intersect(
                shape,
                line,
                visible_only,
                face_visibility,
            )),
            ShapeType::SingleFace(single_face) => BranchIterator::Option2(
                single_face
                    .line_intersect(shape, line, visible_only, face_visibility)
                    .into_iter(),
            ),
            ShapeType::Generic(g) => BranchIterator::Option3(g.line_intersect(
                &shape.faces,
                line,
                visible_only,
                face_visibility,
            )),
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
impl Edge {
    pub fn map<F: Fn(VertIndex) -> VertIndex>(&self, f: F) -> Self {
        Self(f(self.0), f(self.1))
    }
    pub fn contains(&self, vi: VertIndex) -> bool {
        self.0 == vi || self.1 == vi
    }
    pub fn get_line<V: Copy>(&self, shape_verts: &[V]) -> Line<V> {
        Line(shape_verts[self.0], shape_verts[self.1])
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
        faces: Vec<Face<V>>,
        shape_type: ShapeType<V>,
    ) -> Self {
        Self {
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

    pub fn new_single_face(
        verts: Vec<V>,
        edges: Vec<Edge>,
        face: Face<V>,
        subface_vertis: &[Vec<VertIndex>],
    ) -> Self {
        let shape_type =
            ShapeType::SingleFace(SingleFace::new(&verts, face.normal(), subface_vertis, 0));
        Self::new(verts, edges, vec![face], shape_type)
    }

    /// returns the max signed distance to any face plane
    /// for a convex shape, only
    pub fn point_signed_distance(&self, point: V) -> Field {
        self.faces
            .iter()
            .map(|f| f.plane().point_signed_distance(point))
            .fold(Field::NEG_INFINITY, |a, b| match a > b {
                true => a,
                false => b,
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
        self.shape_type.update(&self.verts, &self.faces)
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
        self.shape_type.update(&self.verts, &self.faces)
    }

    pub fn to_generic(&self) -> Self {
        Self::new(
            self.verts.clone(),
            self.edges.clone(),
            self.faces.clone(),
            self.shape_type.to_generic(),
        )
    }
}
// impl<V: VectorTrait> Transformable<V> for Shape<V> {
//     fn transform(&mut self, transformation: Transform<V>) {
//         self.update( &transformation)
//     }
// }
