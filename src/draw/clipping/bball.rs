use crate::vector::{Field, VectorTrait};

pub struct BBall<V> {
    pub pos: V,
    pub radius: Field,
}

impl<V: VectorTrait> BBall<V> {
    pub fn new(verts: &[V], pos: V) -> Self {
        let radius = verts
            .iter()
            .map(|v| v.norm_sq())
            .fold(Field::NAN, Field::max)
            .sqrt();
        Self { pos, radius }
    }
}
