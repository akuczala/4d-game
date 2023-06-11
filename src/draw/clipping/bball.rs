use std::marker::PhantomData;

use specs::{Component,VecStorage};

use crate::components::{Shape, Transform};
use crate::ecs_utils::ModSystem;
use crate::vector::{VectorTrait, Field};

#[derive(Component)]
#[storage(VecStorage)]
pub struct BBall<V: VectorTrait> {
    pub pos: V, pub radius: Field,
}
impl<V: VectorTrait> BBall<V> {
    pub fn new(verts: &Vec<V>, pos: V) -> Self {
        let radius = verts.iter().map(|v| v.norm_sq()).fold(0./0., Field::max).sqrt();
        Self{pos,radius}
    }
}