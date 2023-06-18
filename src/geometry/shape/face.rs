use super::{EdgeIndex,VertIndex,Edge};
use crate::vector::{VectorTrait,Field};
use crate::graphics::colors::Color;
use itertools::Itertools;

use crate::draw::{Texture,TextureMapping};
// TODO: move texture + texture mapping to separate component entirely
#[derive(Clone)]
pub struct Face<V : VectorTrait> {
    pub geometry: FaceGeometry<V>,

    pub edgeis : Vec<EdgeIndex>,
    pub vertis: Vec<VertIndex>

}

#[derive(Clone)]
pub struct FaceGeometry<V: VectorTrait> {
    pub plane: Plane<V>,
    pub center : V,
}


impl<V : VectorTrait> Face<V> {
    pub fn new(edgeis : Vec<EdgeIndex>, normal : V) -> Face<V> {
        let face = Face{
            geometry: FaceGeometry{
                plane: Plane{
                    normal : normal.normalize(), //let's make 100% these are normalized
                    threshold : 0.0,
                },
                center : V::zero(),
            },

            edgeis : edgeis,
            vertis : Vec::new() 

        };

        face
    }
    //compute vertex indices from edge indices and a list of edges
    pub fn calc_vertis(&mut self, edges : &Vec<Edge>) {
        let mut vertis : Vec<VertIndex> = Vec::new();
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
        return &self.geometry.plane
    }
    pub fn normal(&self) -> V {
        return self.plane().normal
    }
    pub fn center(&self) -> V {
        return self.geometry.center
    }

}

use std::fmt;
use crate::geometry::Plane;

impl<V : VectorTrait> fmt::Display for Face<V> {
        // This trait requires `fmt` with this exact signature.
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut out = format!("normal={}, ",self.normal());
            out.push_str("edgeis=[");
            for ei in self.edgeis.iter() {
                out.push_str(&format!("{},",*ei));
            }
            out.push_str("]");
            write!(f,"{}", out)
        }
}