use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{convex::ConvexSubFace, face, subface::SubFace, FaceIndex};

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
}
