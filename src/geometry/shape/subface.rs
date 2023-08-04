use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    geometry::Plane,
    vector::{barycenter_iter, VectorTrait},
};

use super::{face, FaceIndex, VertIndex};

#[derive(Clone, Serialize, Deserialize)]
pub enum SubFace<V> {
    Interior(InteriorSubFace),
    Boundary(BoundarySubFace<V>),
}
impl<V> SubFace<V> {
    pub fn is_face_subface(&self, face_index: FaceIndex) -> bool {
        match self {
            SubFace::Interior(isf) => isf.is_face_subface(face_index),
            SubFace::Boundary(bsf) => bsf.is_face_subface(face_index),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InteriorSubFace {
    pub faceis: (FaceIndex, FaceIndex),
    // consider adding a slot for memoizing plane info
}
impl InteriorSubFace {
    pub fn is_face_subface(&self, face_index: FaceIndex) -> bool {
        self.faceis.0 == face_index || self.faceis.1 == face_index
    }

    /// returns other face index if face_index belongs to subface, otherwise None
    pub fn other_face_index(&self, face_index: FaceIndex) -> Option<FaceIndex> {
        match face_index {
            _ if face_index == self.faceis.0 => Some(self.faceis.1),
            _ if face_index == self.faceis.1 => Some(self.faceis.0),
            _ => None,
        }
    }
}
impl fmt::Display for InteriorSubFace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SubFace({},{})", self.faceis.0, self.faceis.1)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BoundarySubFace<V> {
    pub vertis: Vec<VertIndex>, // list of vertis in each subface
    pub plane: Plane<V>, // this is not used for clipping, but is used for collisions + line intersection (e.g. targeting)
    pub facei: FaceIndex,
}
impl<V> BoundarySubFace<V> {
    pub fn is_face_subface(&self, face_index: FaceIndex) -> bool {
        self.facei == face_index
    }
}
impl<V: VectorTrait> BoundarySubFace<V> {
    pub fn new(vertis: &[VertIndex], shape_verts: &[V], face_normal: V, facei: FaceIndex) -> Self {
        Self {
            vertis: vertis.to_owned(),
            plane: Self::calc_plane(vertis, shape_verts, face_normal),
            facei,
        }
    }
    pub fn update(&mut self, shape_verts: &[V], face_normal: V) {
        self.plane = Self::calc_plane(&self.vertis, shape_verts, face_normal)
    }
    fn calc_plane(vertis: &[VertIndex], shape_verts: &[V], face_normal: V) -> Plane<V> {
        //note: would like to use some of the logic in Plane::calc_plane but here there are differences
        // take D-1 vertices of the subface, then subtract one of these from the others to get
        // D-2 vectors parallel to the subface
        let mut verts = vertis
            .iter()
            .take((V::DIM.unsigned_abs() - 1) as usize)
            .map(|&vi| shape_verts[vi]);
        let v0: V = verts.next().unwrap();
        let parallel_vecs = verts.map(|v| v - v0);
        let mut normal =
            V::cross_product(parallel_vecs.chain(std::iter::once(face_normal))).normalize();
        let shape_center = barycenter_iter(shape_verts.iter());
        if normal.dot(v0 - shape_center) < 0.0 {
            //normal should be pointing outward from center
            normal = -normal;
        }
        let subface_center = barycenter_iter(vertis.iter().map(|&vi| &shape_verts[vi]));
        Plane::from_normal_and_point(normal, subface_center)
    }
}
