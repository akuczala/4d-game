use specs::prelude::*;
use specs::{Component};
use crate::vector::{VectorTrait,MatrixTrait,VecIndex,Field,rotation_matrix};
use super::Face;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Transform<V: VectorTrait>{
    pos: V,
    frame: V::M,
    scale: Field
}
impl<V: VectorTrait> Transform<V> {
    fn translate(&mut self, dpos: V) {
        self.set_pos(self.get_pos() + dpos);
    }
    fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) {
        let rot_mat = rotation_matrix(self.frame[axis1], self.frame[axis2], Some(angle));
        self.frame = self.get_frame().dot(rot_mat);
    }
    fn transform_vec(&self, &vec: V) -> V {
        *self.frame * (*vec * self.scale) + self.pos
    }
    fn transform_verts(&self, verts: &mut Vec<V>, verts_ref: &Vec<V>) {
        for (v,vr) in verts.iter_mut().zip(verts_ref.iter()) {
            *v = self.transform_vec(vr)
        }
    }
    fn transform_face(&self, face: &mut Face<V>) {
        //not affected by scale atm
        face.normal = *self.frame * face.normal_ref;
        face.threshold = face.normal.dot(self.pos);
        face.center = *self.frame * face.center_ref + self.pos;
    }
}