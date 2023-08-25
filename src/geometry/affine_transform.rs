#![allow(dead_code)]
use crate::vector::{Field, MatrixTrait, VectorTrait};

use super::transform::Scaling;

pub struct AffineTransform<V, M> {
    pub pos: V,
    pub frame: M,
}
// todo figure out how to "snap" Affinetransforms to e.g. integer scales, deg of rotation, grid pos

// TODO improve performance by creating fewer structs? or does the compiler do that
impl<V: VectorTrait> AffineTransform<V, V::M> {
    pub fn identity() -> Self {
        Self {
            pos: V::zero(),
            frame: V::M::id(),
        }
    }
    pub fn pos(pos: V) -> Self {
        let mut new = AffineTransform::identity();
        new.pos = pos;
        new
    }
    pub fn new(maybe_pos: Option<V>, maybe_frame: Option<V::M>) -> Self {
        Self {
            pos: maybe_pos.unwrap_or(V::zero()),
            frame: maybe_frame.unwrap_or(V::M::id()),
        }
    }
    pub fn translate(&mut self, pos_delta: V) {
        self.pos = self.pos + pos_delta
    }

    pub fn decompose_rotation_scaling(&self) -> (V::M, Scaling<V>) {
        // i tried using normalize_get_norm + unzip but rust hates me
        let cols: Vec<V> = self.frame.transpose().get_rows();
        let norms: Vec<Field> = cols.iter().map(|v| v.norm()).collect();
        //for n in norms.iter() { println!{":: {}", n}}
        (
            V::M::from_vec_of_vecs(
                &self
                    .frame
                    .transpose()
                    .get_rows()
                    .iter()
                    .zip(norms.iter())
                    .map(|(v, n)| *v / *n)
                    .collect::<Vec<V>>(),
            )
            .transpose(),
            Scaling::Vector(norms.into_iter().collect()),
        )
    }
    pub fn unshear(&self) -> AffineTransform<V, V::M> {
        let (rotation, scaling) = self.decompose_rotation_scaling();
        AffineTransform::new(Some(self.pos), Some(rotation.dot(scaling.get_mat())))
    }

    pub fn transform_vec(&self, &vec: &V) -> V {
        self.frame * vec + self.pos
    }
    pub fn set_transform(&mut self, transform: AffineTransform<V, V::M>) {
        self.pos = transform.pos;
        self.frame = transform.frame;
    }
    //  T1 = A1 v + p1 and T2 compose as affine transformations:
    // T1 T2 v = T1 (A2 v + p2) = A1 (A2 v + p2) + p1 = (A1 A2) v + (A1 p2 + p1)

    pub fn apply_self_on_left(&mut self, transformation: AffineTransform<V, V::M>) {
        let other = transformation;
        self.pos = self.pos + self.frame * other.pos;
        self.frame = self.frame.dot(other.frame);
    }
    pub fn apply_self_on_right(&mut self, transformation: AffineTransform<V, V::M>) {
        let other = transformation;
        self.pos = other.pos + other.frame * self.pos;
        self.frame = other.frame.dot(self.frame);
    }
    pub fn compose(&mut self, transformation: AffineTransform<V, V::M>) {
        self.apply_self_on_left(transformation) //scale composition commutes
    }
    pub fn with_transform(mut self, transformation: AffineTransform<V, V::M>) -> Self {
        self.compose(transformation);
        self
    }
    pub fn with_translation(mut self, pos_delta: V) -> Self {
        self.translate(pos_delta);
        self
    }
}
