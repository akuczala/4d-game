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
    pub fn from_verts(verts: &[V]) -> Self {
        //take smallest and largest components to get bounding box
        let (mut min, mut max) = (verts[0], verts[0]);
        for &v in verts.iter() {
            min = min.zip_map(v, Field::min);
            max = max.zip_map(v, Field::max);
        }
        BBox { min, max }
    }
    pub fn random_point(&self) -> V {
        (self.max - self.min).elmt_mult(V::random()) - self.min
    }
}

pub trait HasBBox<V> {
    fn calc_bbox(&self) -> BBox<V>;
}
