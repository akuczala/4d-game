use super::{Edge, EdgeIndex, VertIndex};
use crate::graphics::colors::Color;
use crate::vector::{Field, VectorTrait};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::draw::{Texture, TextureMapping};
// TODO: move texture + texture mapping to separate component entirely
#[derive(Clone, Serialize, Deserialize)]
pub struct Face<V> {
    pub geometry: FaceGeometry<V>,

    pub edgeis: Vec<EdgeIndex>,
    pub vertis: Vec<VertIndex>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FaceGeometry<V> {
    pub plane: Plane<V>,
    pub center: V,
}

impl<V: VectorTrait> Face<V> {
    pub fn new(edgeis: Vec<EdgeIndex>, normal: V) -> Face<V> {
        Face {
            geometry: FaceGeometry {
                plane: Plane {
                    normal: normal.normalize(), //let's make 100% these are normalized
                    threshold: 0.0,
                },
                center: V::zero(),
            },

            edgeis,
            vertis: Vec::new(),
        }
    }
    //compute vertex indices from edge indices and a list of edges
    pub fn calc_vertis(&mut self, edges: &[Edge]) {
        let mut vertis: Vec<VertIndex> = Vec::new();
        for edgei in self.edgeis.iter() {
            let edge = &edges[*edgei];
            vertis.push(edge.0);
            vertis.push(edge.1);
        }
        //this is probably inefficient
        for verti in vertis.iter().unique() {
            self.vertis.push(*verti);
        }
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
