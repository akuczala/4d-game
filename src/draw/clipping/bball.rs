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

#[derive(Default)]
pub struct UpdateBBallSystem<V: VectorTrait> {
    pub ph: PhantomData<V>,
    pub modified: BitSet,
    pub reader_id: Option<ReaderId<ComponentEvent>>
}


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
        self.modified.clear();
        let events = read_shape.channel().read(self.reader_id.as_mut().unwrap());
        for event in events {
            match event {
                ComponentEvent::Modified(id) => {self.modified.add(*id);},
                _ => (),
            }
        }
        for (_, shape, transform, bball) in (&self.modified, &read_shape, &read_transform, &mut write_bball).join() {
            *bball =  BBall::new(&shape.verts, transform.pos);
            println!("bball {}", bball.pos);
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.reader_id = Some(
            WriteStorage::<Shape<V>>::fetch(&world).register_reader()
        );
    }

    
}