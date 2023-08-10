use crate::vector::{Field, VectorTrait};

//axis-aligned bounding box
#[derive(Debug, Clone)]
pub struct BBox<V> {
    pub min: V,
    pub max: V,
}

impl<V: VectorTrait> BBox<V> {
    #[allow(dead_code)]
    pub fn max_length(&self) -> Field {
        (self.max - self.min).fold(Some(0.0), |x, y| match x > y {
            true => x,
            false => y,
        })
    }
    #[allow(dead_code)]
    pub fn center(&self) -> V {
        (self.max + self.min) / 2.0
    }
}

pub trait HasBBox<V> {
    fn calc_bbox(&self) -> BBox<V>;
}
