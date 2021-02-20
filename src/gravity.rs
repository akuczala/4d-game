use std::marker::PhantomData;
use specs::prelude::*;
use specs::{System,SystemData,WriteStorage,Read,Component,VecStorage};
use crate::vector::VectorTrait;
use crate::components::MoveNext;
use crate::input::Input;

#[derive(Component,Clone,Copy)]
#[storage(VecStorage)]
pub struct Velocity<V: VectorTrait>(pub V);
impl<V: VectorTrait> Default for Velocity<V> {
    fn default() -> Self {
        Self(V::zero())
    }
}

pub struct PlayerGravitySystem<V>(pub PhantomData<V>);

impl<'a, V : VectorTrait> System<'a> for PlayerGravitySystem<V> {

    type SystemData = (
        Read<'a,Input>,
        WriteStorage<'a,MoveNext<V>>,
        WriteStorage<'a,Velocity<V>>
    );

    fn run(&mut self, (input, mut write_move_next, mut write_velocity) : Self::SystemData) {
        for (move_next, velocity) in (&mut write_move_next, &mut write_velocity).join() {
            let dt = input.get_dt();
            let gvec = -V::one_hot(1)*dt;
            velocity.0 = velocity.0 + gvec;
            match move_next {
                MoveNext{next_dpos: Some(next_dpos), can_move: Some(true)} => {
                    move_next.next_dpos = Some(*next_dpos + velocity.0*dt);
                },
                MoveNext{next_dpos: None,can_move: Some(true)} => {*move_next = MoveNext{next_dpos: Some(velocity.0*dt), can_move: Some(true)}}
                _ => (),
            }
        }
    }
}