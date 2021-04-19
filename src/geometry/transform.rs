use specs::prelude::*;
use specs::{Component};
use crate::vector::{VectorTrait,MatrixTrait,VecIndex,Field,rotation_matrix};

pub trait Transformable<V: VectorTrait> {
    fn with_transform(mut self, transformation: Transform<V>) -> Self
        where Self: std::marker::Sized {
        self.transform(transformation);
        self
    }
    fn transform(&mut self, transformation: Transform<V>);
    fn translate(&mut self, pos: V) {
        self.transform(Transform::pos(pos))
    }
    fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) {
        self.transform(Transform::identity().with_rotation(axis1, axis2, angle))
    }
    fn stretch(&mut self, scale: Scaling<V>) {
        self.transform(Transform::new(None, None, Some(scale)))
    }
    fn with_translation(mut self, pos: V) -> Self
        where Self: std::marker::Sized {
        self.translate(pos); self
    }
    fn with_rotation(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self
        where Self: std::marker::Sized {
        self.rotate(axis1, axis2, angle); self
    }
    fn with_rotation_about(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field, point: V) -> Self
        where Self: std::marker::Sized {
        self.with_transform(Transform::identity().with_rotation_about(axis1, axis2, angle,point))
    }
}
#[derive(Copy,Clone)]
pub enum Scaling<V: VectorTrait> {
    Scalar(Field),
    Vector(V),
}
impl<V: VectorTrait> Scaling<V> {
    fn unit() -> Self {
        Self::Scalar(1.0)
    }
    fn scale_vec(&self, vec: V) -> V {
        match self {
            Self::Scalar(s) => vec*(*s),
            Self::Vector(ref v) => v.elmt_mult(vec)
        }
    }
    fn compose(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (Self::Scalar(ref s1),Self::Scalar(ref s2)) => Self::Scalar(*s1**s2),
            (Self::Vector(ref v1),Self::Scalar(ref s2)) => Self::Vector(*v1**s2),
            (Self::Scalar(ref s1), Self::Vector(ref v2)) => Self::Vector(*v2**s1),
            (Self::Vector(_), Self::Vector(ref v2)) => Self::Vector(self.scale_vec(*v2)),
        }
    }
}
#[derive(Component,Clone,Copy)]
#[storage(VecStorage)]
pub struct Transform<V: VectorTrait>{
    pub pos: V,
    pub frame: V::M,
    pub scale: Scaling<V>,
}
impl<V: VectorTrait> Transform<V> {
    pub fn identity() -> Self {
        Self{
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
    pub fn new(maybe_pos: Option<V>, maybe_frame: Option<V::M>, maybe_scale: Option<Scaling<V>>) -> Self {
        Self {
            pos: maybe_pos.unwrap_or(V::zero()),
            frame: maybe_frame.unwrap_or(V::M::id()),
            scale: maybe_scale.unwrap_or(Scaling::unit())
        }
    }
    pub fn translate(&mut self, pos_delta: V) {
        self.pos = self.pos + pos_delta
    }
    pub fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) {
        let rot_mat = rotation_matrix(self.frame[axis1], self.frame[axis2], Some(angle));
        self.frame = self.frame.dot(rot_mat);
    }
    pub fn rotate_about(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field, pos: V) {
        let rot_mat = rotation_matrix(self.frame[axis1], self.frame[axis2], Some(angle));
        self.frame = self.frame.dot(rot_mat);
        self.pos = self.pos - rot_mat * self.pos
    }
    pub fn stretch(&mut self, scale: Scaling<V>) {
        self.scale = self.scale.compose(&scale)
    }
    pub fn transform_vec(&self, &vec: &V) -> V {
        self.frame * (self.scale.scale_vec(vec)) + self.pos
    }
    pub fn set_transform(&mut self, transform: Transform<V>) {
        self.pos = transform.pos;
        self.frame = transform.frame;
        self.scale = transform.scale;
    }
    //transformations T1 v = R1 S1 v + p1 and T2 compose as
    //T1 T2 v = R1 R2 S1 S2 v + (p1 + p2)
    pub fn compose(&mut self, transformation: Transform<V>) {
        let other = transformation;
        self.pos = self.pos + other.pos;
        self.frame = self.frame.dot(other.frame);
        self.scale = self.scale.compose(&other.scale);
    }
    pub fn with_transform(mut self, transformation: Transform<V>) -> Self {
        self.compose(transformation); self
    }
    pub fn with_translation(mut self, pos_delta: V) -> Self {
        self.translate(pos_delta); self
    }
    pub fn with_rotation(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self {
        self.rotate(axis1, axis2, angle); self
    }
    pub fn with_rotation_about(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field, point: V) -> Self {
        self.rotate_about(axis1, axis2, angle, point); self
    }

}

