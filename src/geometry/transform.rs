use crate::vector::{rotation_matrix, Field, MatrixTrait, Vec4, VecIndex, VectorTrait};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::Component;

pub trait Transformable<V: VectorTrait> {
    fn with_transform(mut self, transformation: Transform<V, V::M>) -> Self
    where
        Self: std::marker::Sized,
    {
        self.transform(transformation);
        self
    }
    fn transform(&mut self, transformation: Transform<V, V::M>);
    fn translate(&mut self, pos: V) {
        self.transform(Transform::pos(pos))
    }
    fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) {
        self.transform(Transform::identity().with_rotation(axis1, axis2, angle))
    }
    fn scale(&mut self, scale: Scaling<V>) {
        self.transform(Transform::new(None, None, Some(scale)))
    }
    fn with_translation(mut self, pos: V) -> Self
    where
        Self: std::marker::Sized,
    {
        self.translate(pos);
        self
    }
    fn with_rotation(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self
    where
        Self: std::marker::Sized,
    {
        self.rotate(axis1, axis2, angle);
        self
    }
    fn with_rotation_about(self, axis1: VecIndex, axis2: VecIndex, angle: Field, point: V) -> Self
    where
        Self: std::marker::Sized,
    {
        self.with_transform(Transform::identity().with_rotation_about(axis1, axis2, angle, point))
    }
    fn with_scale(mut self, scale: Scaling<V>) -> Self
    where
        Self: std::marker::Sized,
    {
        self.scale(scale);
        self
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Scaling<V> {
    Scalar(Field),
    Vector(V),
}
impl<V: VectorTrait> Scaling<V> {
    pub fn unit() -> Self {
        Self::Scalar(1.0)
    }
    pub fn scale_vec(&self, vec: V) -> V {
        match self {
            Self::Scalar(s) => vec * (*s),
            Self::Vector(ref v) => v.elmt_mult(vec),
        }
    }
    fn compose(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (Self::Scalar(ref s1), Self::Scalar(ref s2)) => Self::Scalar(*s1 * *s2),
            (Self::Vector(ref v1), Self::Scalar(ref s2)) => Self::Vector(*v1 * *s2),
            (Self::Scalar(ref s1), Self::Vector(ref v2)) => Self::Vector(*v2 * *s1),
            (Self::Vector(_), Self::Vector(ref v2)) => Self::Vector(self.scale_vec(*v2)),
        }
    }
    pub(crate) fn get_vec(&self) -> V {
        match self {
            Scaling::Scalar(s) => V::ones() * *s,
            Scaling::Vector(v) => *v,
        }
    }
    pub(crate) fn get_mat(&self) -> V::M {
        V::M::diag(self.get_vec())
    }
}

// There is no proper subgroup of GL(n) generated by rotations + axis scaling (ortho + diag matrices)
// Proof: we can write any real matrix A as U H, where U in O(n) and H is symmetric.
// We can write H = V.T D V for some diagonal D, ortho V. So A = U V.T D V, a composition of ortho +
// diag matrices (I probably could have just used SVD)

// we could also create a more restrictive trait + struct RigidTransform, that requires the matrix A to be
// orthogonal. This might help us where the code assumes "frame" is orthogonal

// todo: there is a nice way to "compose" rotations and scalings, see blender
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Transform<V, M> {
    pub pos: V,
    pub frame: M,
    pub scale: Scaling<V>,
}

// todo figure out how to "snap" transforms to e.g. integer scales, deg of rotation, grid pos

// TODO improve performance by creating fewer structs? or does the compiler do that
impl<V: VectorTrait> Transform<V, V::M> {
    pub fn identity() -> Self {
        Self {
            pos: V::zero(),
            frame: V::M::id(),
            scale: Scaling::unit(),
        }
    }
    pub fn pos(pos: V) -> Self {
        let mut new = Transform::identity();
        new.pos = pos;
        new
    }
    pub fn new(
        maybe_pos: Option<V>,
        maybe_frame: Option<V::M>,
        maybe_scale: Option<Scaling<V>>,
    ) -> Self {
        Self {
            pos: maybe_pos.unwrap_or(V::zero()),
            frame: maybe_frame.unwrap_or(V::M::id()),
            scale: maybe_scale.unwrap_or(Scaling::unit()),
        }
    }
    pub fn translate(&mut self, pos_delta: V) {
        self.pos = self.pos + pos_delta
    }

    pub fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) {
        let rot: Transform<V, V::M> = Transform::new(
            None,
            Some(rotation_matrix(
                self.frame[axis1],
                self.frame[axis2],
                Some(angle),
            )),
            None,
        );
        self.frame = rot.frame.dot(self.frame)
    }

    //todo update
    pub fn rotate_about(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field, pos: V) {
        let rot_mat = rotation_matrix(self.frame[axis1], self.frame[axis2], Some(angle));
        self.frame = self.frame.dot(rot_mat);
        self.pos = self.pos - rot_mat * pos
    }
    pub fn scale(&mut self, scale: Scaling<V>) {
        self.scale = self.scale.compose(&scale);
    }
    pub fn transform_vec(&self, &vec: &V) -> V {
        self.frame * (self.scale.scale_vec(vec)) + self.pos
    }
    pub fn set_transform(&mut self, transform: Transform<V, V::M>) {
        self.pos = transform.pos;
        self.frame = transform.frame;
        self.scale = transform.scale;
    }
    //BACK TO transformations T1 v = R1 S1 v + p1 and T2 composed as
    //T1 T2 v = R1 R2 S1 S2 v + (p1 + p2)
    // are NOT composed like affine transformations

    pub fn compose(&mut self, transformation: Transform<V, V::M>) {
        self.pos = self.pos + transformation.pos;
        self.frame = self.frame.dot(transformation.frame);
        self.scale = self.scale.compose(&transformation.scale); //scale composition commutes
    }
    pub fn with_transform(mut self, transformation: Transform<V, V::M>) -> Self {
        self.compose(transformation);
        self
    }
    pub fn with_translation(mut self, pos_delta: V) -> Self {
        self.translate(pos_delta);
        self
    }
    pub fn with_rotation(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self {
        self.rotate(axis1, axis2, angle);
        self
    }
    pub fn with_rotation_about(
        mut self,
        axis1: VecIndex,
        axis2: VecIndex,
        angle: Field,
        point: V,
    ) -> Self {
        self.rotate_about(axis1, axis2, angle, point);
        self
    }
    pub fn with_scale(mut self, scaling: Scaling<V>) -> Self {
        self.scale(scaling);
        self
    }
}
