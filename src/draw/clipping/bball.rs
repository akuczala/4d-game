use std::marker::PhantomData;

use specs::prelude::*;
use specs::{Component,VecStorage};

use crate::components::{Shape, Transform};
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


pub struct UpdateBBallSystem<V: VectorTrait>(pub PhantomData<V>);

impl<'a, V: VectorTrait> System<'a> for UpdateBBallSystem<V> {

    type SystemData = (
        ReadStorage<'a, Shape<V>>,
        ReadStorage<'a, Transform<V>>,
        WriteStorage<'a, BBall<V>>
    );

    fn run(
        &mut self, 
        (
            read_shape,
            read_transform,
            mut write_bball
        ): Self::SystemData
    ) {
        for (shape, transform, bball) in (&read_shape, &read_transform, &mut write_bball).join() {
            *bball =  BBall::new(&shape.verts, transform.pos);
        }
    }

    
}