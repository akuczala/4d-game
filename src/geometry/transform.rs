use specs::prelude::*;
use specs::{Component};
use crate::vector::{VectorTrait,MatrixTrait,VecIndex,Field,rotation_matrix};
use super::Face;

pub trait Transformable<V: VectorTrait> {
    fn set_identity(self) -> Self;
    fn transform(self, transformation: Transform<V>) -> Self;
    // fn with_pos(self, pos: V) -> Self
    // where Self: std::marker::Sized {
    //     self.set_identity().with_translation(pos)
    // }
    fn with_translation(mut self, pos: V) -> Self
        where Self: std::marker::Sized {

        self.transform(Transform::identity().with_translation(pos))
    }
    fn with_rotation(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self
        where Self: std::marker::Sized {
        self.transform(Transform::identity().with_rotation(axis1, axis2, angle))
    }
}

#[derive(Component,Clone)]
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
    pub fn translate(&mut self, dpos: V) {
        self.set_pos(self.pos + dpos);
    }
    pub fn set_pos(&mut self, pos: V) {
        self.pos = pos;
    }
    pub fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) {
        let rot_mat = rotation_matrix(self.frame[axis1], self.frame[axis2], Some(angle));
        self.frame = self.frame.dot(rot_mat);
    }
    pub fn transform_vec(&self, vec: &V) -> V {
        self.frame * (*vec * self.scale) + self.pos
    }
}
impl<V: VectorTrait> Transformable<V> for Transform<V> {
    fn set_identity(mut self) -> Self {
        Self::identity()
    }
    fn transform(mut self, transformation: Transform<V>) -> Self {
        let other = transformation;
        self.pos = self.pos + other.pos;
        self.frame = self.frame.dot(other.frame);
        self.scale = self.scale * other.scale;
        self
    }
    fn with_translation(mut self, dpos: V) -> Self {
        self.translate(dpos); self
    }
    fn with_rotation(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self {
        self.rotate(axis1, axis2, angle); self
    }

}

