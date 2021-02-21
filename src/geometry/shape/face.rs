use super::{EdgeIndex,VertIndex,Edge};
use crate::vector::{VectorTrait,Field};
use crate::colors::Color;
use itertools::Itertools;

use crate::draw::{Texture,TextureMapping};

#[derive(Clone)]
pub struct Face<V : VectorTrait> {
    pub normal : V, 
    pub normal_ref : V,

    pub center : V,
    pub center_ref : V,

    pub threshold : Field,

    pub texture : Texture<V::SubV>,
    pub texture_mapping : TextureMapping,
    pub visible : bool,

    pub edgeis : Vec<EdgeIndex>,
    pub vertis: Vec<VertIndex>

}

impl<V : VectorTrait> Face<V> {
    pub fn new(edgeis : Vec<EdgeIndex>, normal : V) -> Face<V> {
        let face = Face{
            normal : normal.normalize(), //let's make 100% these are normalized
            normal_ref : normal.normalize(),

            center : V::zero(),
            center_ref : V::zero(),

            threshold : 0.0,

            //change texture to reference
            texture : Default::default(),
            texture_mapping : Default::default(),
            visible: true,

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
    pub fn update_visibility(&mut self,camera_pos : V)
    {
        self.visible = self.normal.dot(self.center - camera_pos) < 0.0;
    }
    pub fn set_color(&mut self, color : Color) {
        take_mut::take(&mut self.texture,|tex| tex.set_color(color));
    }
    pub fn set_texture(&mut self, texture: Texture<V::SubV>, texture_mapping: TextureMapping) {
        self.texture = texture;
        self.texture_mapping = texture_mapping;
    }
    pub fn with_texture(mut self, texture: Texture<V::SubV>, texture_mapping: TextureMapping) -> Self {
        self.set_texture(texture, texture_mapping);
        self
    }

}

use std::fmt;
impl<V : VectorTrait> fmt::Display for Face<V> {
        // This trait requires `fmt` with this exact signature.
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut out = format!("n_ref={}, ",self.normal_ref);
            out.push_str("edgeis=[");
            for ei in self.edgeis.iter() {
                out.push_str(&format!("{},",*ei));
            }
            out.push_str("]");
            write!(f,"{}", out)
        }
}