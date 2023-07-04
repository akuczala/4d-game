use crate::components::MoveNext;
use crate::ecs_utils::Componentable;
use crate::input::Input;
use crate::vector::VectorTrait;
use specs::prelude::*;
use specs::{Read, System, WriteStorage};
use std::marker::PhantomData;

pub struct PlayerGravitySystem<V>(pub PhantomData<V>);

impl<'a, V: VectorTrait + Componentable> System<'a> for PlayerGravitySystem<V> {
    type SystemData = (
        Read<'a, Input>,
        WriteStorage<'a, MoveNext<V>>, //gravity should apply even if the player isn't moving, but debug for now
    );

    fn run(&mut self, (input, mut write_move_next): Self::SystemData) {
        for move_next in (&mut write_move_next).join() {
            let gvec = -V::one_hot(1) * input.get_dt() * 0.0;
            match move_next {
                MoveNext {
                    next_dpos: Some(next_dpos),
                    can_move: Some(true),
                } => {
                    move_next.next_dpos = Some(*next_dpos + gvec);
                }
                MoveNext {
                    next_dpos: None,
                    can_move: Some(true),
                } => {
                    *move_next = MoveNext {
                        next_dpos: Some(gvec),
                        can_move: Some(true),
                    }
                }
                _ => (),
            }
        }
    }
}
