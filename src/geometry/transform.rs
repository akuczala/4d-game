use specs::prelude::*;
use specs::{Component};
use crate::vector::{VectorTrait,MatrixTrait,VecIndex,Field,rotation_matrix};
use super::Face;
use std::convert::identity;

pub trait Transformable<V: VectorTrait> {
    fn set_identity(self) -> Self;
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
    fn with_rotation(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self
        where Self: std::marker::Sized {
        self.with_transform(Transform::identity().with_rotation(axis1, axis2, angle))
    }
}

#[derive(Component,Clone,Copy)]
#[storage(VecStorage)]
pub struct Transform<V: VectorTrait>{
    pub pos: V,
    pub frame: V::M,
    pub scale: Field
}
impl<V: VectorTrait> Transform<V> {
    pub fn identity() -> Self {
        Self{
            pos: V::zero(),
            frame: V::M::id(),
            scale: 1.0,
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
    pub fn transform_vec(&self, vec: &V) -> V {
        self.frame * (*vec * self.scale) + self.pos
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
    fn set_identity(mut self) -> Self {
        Self::identity()
    }
    fn transform(&mut self, transformation: Transform<V>) {
        let other = transformation;
        self.pos = self.pos + other.pos;
        self.frame = self.frame.dot(other.frame);
        self.scale = self.scale * other.scale;
    }
    fn with_rotation(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self {
        self.rotate(axis1, axis2, angle); self
    }

}

