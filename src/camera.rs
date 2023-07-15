use crate::components::{Transform, Transformable};
use crate::constants::{PI, ANG_SPEED, MAX_TILT};
use crate::geometry::Plane;
use crate::vector::{rotation_matrix, Field, MatrixTrait, VecIndex, VectorTrait};

pub struct Camera<V, M> {
    pub heading: M,
    pub plane: Plane<V>,
}
impl<V: VectorTrait> Camera<V, V::M> {

    pub fn new(transform: &Transform<V, V::M>) -> Camera<V, V::M> {
        Camera {
            heading: V::M::id(),
            plane: Plane {
                normal: V::one_hot(-1),
                threshold: V::one_hot(-1).dot(transform.pos),
            },
        }
    }
    pub fn look_at(&mut self, transform: &mut Transform<V, V::M>, point: &V) {
        transform.frame = rotation_matrix(*point - transform.pos, V::one_hot(-1), None);
        self.update(transform);
    }
    pub fn update_plane(&mut self, transform: &Transform<V, V::M>) {
        self.plane = Plane {
            normal: transform.frame[-1],
            threshold: transform.frame[-1].dot(transform.pos),
        }
    }
    pub fn update(&mut self, transform: &Transform<V, V::M>) {
        self.update_plane(transform);
    }
}
