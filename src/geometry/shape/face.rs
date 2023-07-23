use super::{Edge, EdgeIndex, Shape, VertIndex};
use crate::graphics::colors::Color;
use crate::vector::{self, Field, VectorTrait};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::draw::{Texture, TextureMapping};

#[derive(Clone)]
pub struct FaceBuilder<'a, V> {
    shape_verts: &'a [V],
    shape_edges: &'a [Edge],
}
impl<'a, V: VectorTrait> FaceBuilder<'a, V> {
    pub fn new(shape_verts: &'a [V], shape_edges: &'a [Edge]) -> Self {
        Self {
            shape_verts,
            shape_edges,
        }
    }
    pub fn build(&'a self, edgeis: Vec<EdgeIndex>, normal: V, two_sided: bool) -> Face<V> {
        Face::new(
            self.shape_verts,
            self.shape_edges,
            edgeis,
            normal,
            two_sided,
        )
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Face<V> {
    pub geometry: FaceGeometry<V>,

    pub edgeis: Vec<EdgeIndex>,
    pub vertis: Vec<VertIndex>,
    pub two_sided: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FaceGeometry<V> {
    pub plane: Plane<V>,
    pub center: V,
}

impl<V: VectorTrait> Face<V> {
    pub fn new(
        shape_verts: &[V],
        shape_edges: &[Edge],
        edgeis: Vec<EdgeIndex>,
        normal: V,
        two_sided: bool,
    ) -> Self {
        let vertis = Self::calc_vertis(&edgeis, shape_edges);
        let face_verts = vertis.iter().map(|verti| shape_verts[*verti]).collect();
        let center = vector::barycenter(&face_verts);
        Self {
            geometry: FaceGeometry {
                plane: Plane::from_normal_and_point(normal.normalize(), center),
                center,
            },
            edgeis,
            vertis,
            two_sided,
        }
    }
    //compute vertex indices from edge indices and a list of edges
    pub fn calc_vertis(edgeis: &[EdgeIndex], edges: &[Edge]) -> Vec<VertIndex> {
        let mut vertis: Vec<VertIndex> = Vec::new();
        for edgei in edgeis.iter() {
            let edge = &edges[*edgei];
            vertis.push(edge.0);
            vertis.push(edge.1);
        }
        vertis.into_iter().unique().collect()
    }
    // convenience getters
    pub fn plane(&self) -> &Plane<V> {
        &self.geometry.plane
    }
    pub fn normal(&self) -> V {
        self.plane().normal
    }
    pub fn center(&self) -> V {
        self.geometry.center
    }
}

use crate::geometry::Plane;
use std::fmt;

impl<V: VectorTrait> fmt::Display for Face<V> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = format!("normal={}, ", self.normal());
        out.push_str("edgeis=[");
        for ei in self.edgeis.iter() {
            out.push_str(&format!("{},", *ei));
        }
        out.push(']');
        write!(f, "{}", out)
    }
}
