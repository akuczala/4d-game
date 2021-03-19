use specs::prelude::*;
use specs::{Component};
use crate::vector::{VectorTrait,MatrixTrait,VecIndex,Field,rotation_matrix};

pub trait Transformable<V: VectorTrait> {
    fn with_transform(mut self, transformation: Transform<V>) -> Self
        where Self: std::marker::Sized {
        self.transform(transformation);
        self
    }
    // fn with_set_transform(mut self, transformation: Transform<V>) -> Self
    //     where Self: std::marker::Sized {
    //     self.set_transform(transformation);
    //     self
    // }
    // fn with_pos(self, pos: V) -> Self
    // where Self: std::marker::Sized {
    //     self.set_identity().with_translation(pos)
    // }
    //fn set_transform(&mut self, transform: Transform<V>);
    //fn set_pos(&mut self, pos: V);
    fn transform(&mut self, transformation: Transform<V>);
    fn with_translation(mut self, pos: V) -> Self
        where Self: std::marker::Sized {
        self.translate(pos); self
    }
    fn translate(&mut self, pos: V) {
        self.transform(Transform::pos(pos))
    }
    fn with_rotation(self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self
        where Self: std::marker::Sized {
        self.with_transform(Transform::identity().with_rotation(axis1, axis2, angle))
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
    pub fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) {
        let rot_mat = rotation_matrix(self.frame[axis1], self.frame[axis2], Some(angle));
        self.frame = self.frame.dot(rot_mat);
    }
    pub fn transform_vec(&self, &vec: &V) -> V {
        self.frame * (self.scale.scale_vec(vec)) + self.pos
    }
    pub fn set_transform(&mut self, transform: Transform<V>) {
        self.pos = transform.pos;
        self.frame = transform.frame;
        self.scale = transform.scale;
    }
    pub fn set_pos(&mut self, pos: V) {
        self.pos = pos;
    }
}
impl<V: VectorTrait> Transformable<V> for Transform<V> {
    fn transform(&mut self, transformation: Transform<V>) {
        let other = transformation;
        self.pos = self.pos + other.pos;
        self.frame = self.frame.dot(other.frame);
        self.scale = self.scale.compose(&other.scale);
    }
    fn with_rotation(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self {
        self.rotate(axis1, axis2, angle); self
    }

}

