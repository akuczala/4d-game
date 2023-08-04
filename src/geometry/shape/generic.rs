use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    constants::ZERO,
    geometry::{line_plane_intersect, Line, Plane},
    utils::partial_max,
    vector::{Field, VectorTrait},
};

use super::{convex::ConvexSubFace, face, subface::SubFace, Face, FaceIndex, ShapeTypeTrait};

fn add_pair(
    map: &mut HashMap<FaceIndex, HashSet<usize>>,
    face_index: FaceIndex,
    subface_index: usize,
) {
    if let Some(set) = map.get_mut(&face_index) {
        set.insert(subface_index);
    } else {
        let mut set = HashSet::new();
        set.insert(subface_index);
        map.insert(face_index, set);
    }
}

fn map_to_vecs(map: HashMap<FaceIndex, HashSet<usize>>) -> Vec<Vec<usize>> {
    (0..map.len())
        .map(|face_index| map.get(&face_index).unwrap().iter().cloned().collect_vec())
        .collect_vec()
}

fn build_face_subface_map<V>(subfaces: &[SubFace<V>]) -> HashMap<FaceIndex, HashSet<usize>> {
    let mut face_subface_map: HashMap<FaceIndex, HashSet<usize>> = HashMap::new();
    for (subface_index, subface) in subfaces.iter().enumerate() {
        match subface {
            SubFace::Interior(ConvexSubFace { faceis }) => {
                add_pair(&mut face_subface_map, faceis.0, subface_index);
                add_pair(&mut face_subface_map, faceis.1, subface_index);
            }
            SubFace::Boundary(sf) => add_pair(&mut face_subface_map, sf.facei, subface_index),
        }
    }
    face_subface_map
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GenericShapeType<V> {
    pub subfaces: Vec<SubFace<V>>,
    pub face_subfaces: Vec<Vec<usize>>,
}
impl<V: Clone> GenericShapeType<V> {
    pub fn new(subfaces: &[SubFace<V>]) -> Self {
        let face_subfaces = map_to_vecs(build_face_subface_map(subfaces));
        Self {
            subfaces: subfaces.iter().cloned().collect_vec(),
            face_subfaces,
        }
    }
    pub fn get_face_subfaces(&self, face_index: FaceIndex) -> impl Iterator<Item = &SubFace<V>> {
        self.face_subfaces[face_index]
            .iter()
            .map(|&subface_index| &self.subfaces[subface_index])
    }
}
impl<V: VectorTrait> GenericShapeType<V> {
    pub fn line_intersect(
        &self,
        faces: &[Face<V>],
        line: &Line<V>,
        visible_only: bool,
        face_visibility: &[bool],
    ) -> Vec<V> {
        faces
            .iter()
            .zip(face_visibility.iter())
            .enumerate()
            .filter_map(|(face_index, (face, visible))| {
                (!visible_only || *visible)
                    .then(|| line_plane_intersect(line, face.plane()))
                    .flatten()
                    .and_then(|p| {
                        (max_subface_dist(faces, face_index, self.get_face_subfaces(face_index), p)
                            < 0.0)
                            .then_some(p)
                    })
            })
            .collect()
    }
}

pub fn max_subface_dist<'a, V, I>(
    shape_faces: &[Face<V>],
    face_index: FaceIndex,
    subfaces: I,
    pos: V,
) -> Field
where
    V: VectorTrait + 'a,
    I: Iterator<Item = &'a SubFace<V>>,
{
    partial_max(
        subfaces.map(|subface| subface_signed_distance(shape_faces, face_index, subface, pos)),
    )
    .unwrap()
}

pub fn subface_signed_distance<V: VectorTrait>(
    shape_faces: &[Face<V>],
    face_index: FaceIndex,
    subface: &SubFace<V>,
    pos: V,
) -> Field {
    match subface {
        SubFace::Interior(isf) => {
            interior_subface_plane(shape_faces, face_index, isf).point_signed_distance(pos)
        }
        SubFace::Boundary(bsf) => bsf.plane.point_signed_distance(pos),
    }
}

pub fn subface_plane<V: VectorTrait>(
    shape_faces: &[Face<V>],
    face_index: FaceIndex,
    subface: &SubFace<V>,
) -> Plane<V> {
    match subface {
        SubFace::Interior(isf) => interior_subface_plane(shape_faces, face_index, isf),
        SubFace::Boundary(bsf) => bsf.plane.clone(),
    }
}

/// return plane of adjoining face
fn interior_subface_plane<V: VectorTrait>(
    shape_faces: &[Face<V>],
    face_index: FaceIndex,
    interior_subface: &ConvexSubFace,
) -> Plane<V> {
    let (face_0, face_1) = interior_subface.faceis;
    let plane = shape_faces[if face_index == face_0 { face_1 } else { face_0 }]
        .plane()
        .clone();
    if plane.point_signed_distance(shape_faces[face_index].center()) < ZERO {
        plane
    } else {
        plane.flip_normal()
    }
}
