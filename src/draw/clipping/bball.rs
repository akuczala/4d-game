
use crate::vector::{VectorTrait, Field};

pub struct BBall<V> {
    pub pos: V, pub radius: Field,
}

impl<V: VectorTrait> BBall<V> {
    pub fn new(verts: &Vec<V>, pos: V) -> Self {
        let radius = verts.iter().map(|v| v.norm_sq()).fold(0./0., Field::max).sqrt();
        Self{pos,radius}
    }
}