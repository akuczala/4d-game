use std::marker::PhantomData;
use specs::prelude::*;
use specs::{System,WriteStorage,Read};
use crate::vector::VectorTrait;
use crate::components::MoveNext;
use crate::input::Input;


pub struct PlayerGravitySystem<V>(pub PhantomData<V>);

impl<'a, V : VectorTrait> System<'a> for PlayerGravitySystem<V> {

    type SystemData = (
        Read<'a,Input>,
        WriteStorage<'a,MoveNext<V>>, //gravity should apply even if the player isn't moving, but debug for now
    );

    fn run(&mut self, (input, mut write_move_next) : Self::SystemData) {
        for move_next in (&mut write_move_next).join() {
            let gvec = -V::one_hot(1)*input.get_dt()*0.0;
            match move_next {
                MoveNext{next_dpos: Some(next_dpos), can_move: Some(true)} => {
                    move_next.next_dpos = Some(*next_dpos + gvec);
                },
                MoveNext{next_dpos: None,can_move: Some(true)} => {*move_next = MoveNext{next_dpos: Some(gvec), can_move: Some(true)}}
                _ => (),
            }
        }
    }
}