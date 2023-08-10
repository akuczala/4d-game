use serde::{Deserialize, Serialize};

use super::face::FaceBuilder;
use super::subface::BoundarySubFace;
use super::{Face, FaceIndex, Shape, VertIndex};

use crate::geometry::{line_plane_intersect, Line, Plane};

use crate::vector::{Field, VectorTrait};

#[derive(Clone, Serialize, Deserialize)]
pub struct SingleFace<V> {
    pub subfaces: Vec<BoundarySubFace<V>>,
}
impl<V: VectorTrait> SingleFace<V> {
    pub fn new(
        shape_verts: &[V],
        face_normal: V,
        subface_vertis: &[Vec<VertIndex>],
        face_index: FaceIndex,
    ) -> Self {
        Self {
            subfaces: subface_vertis
                .iter()
                .map(|vertis| BoundarySubFace::new(vertis, shape_verts, face_normal, face_index))
                .collect(),
        }
    }
    pub fn update(&mut self, shape_vers: &[V], shape_faces: &[Face<V>]) {
        for subface in self.subfaces.iter_mut() {
            subface.update(shape_vers, shape_faces[subface.facei].normal())
        }
    }
    //returns points of intersection with shape
    pub fn line_intersect(
        &self,
        shape: &Shape<V>,
        line: &Line<V>,
        visible_only: bool,
        face_visibility: &[bool],
    ) -> Option<V> {
        let face = &shape.faces[0];
        (!visible_only || face_visibility[0])
            .then(|| line_plane_intersect(line, face.plane()))
            .flatten()
            .and_then(|p| (self.subface_normal_distance(p).1 < 0.0).then_some(p))
    }
    // returns distance to nearest subface plane
    pub fn subface_normal_distance(&self, pos: V) -> (V, Field) {
        let (closest_subshape_plane, distance) =
            Plane::point_normal_distance(pos, self.subfaces.iter().map(|sf| &sf.plane)).unwrap();
        (closest_subshape_plane.normal, distance)
    }
}
