use crate::cleanup::DeletedEntities;
use crate::components::*;
use crate::ecs_utils::Componentable;
use crate::geometry::shape::RefShapes;
use crate::input::Input;
use crate::vector::{Field, VectorTrait};
use core::marker::PhantomData;
use specs::prelude::*;
use specs::{Component, VecStorage};

#[derive(Default, Debug)]
pub struct CoinsCollected(pub u32);

pub struct Coin;

const SPIN_SPEED: Field = 2.0;

pub struct CoinSpinningSystem<V>(pub PhantomData<V>);
impl<'a, V> System<'a> for CoinSpinningSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadStorage<'a, Coin>,
        ReadExpect<'a, Input>,
        WriteStorage<'a, Transform<V, V::M>>,
    );

    fn run(&mut self, (read_coin, input, mut write_transform): Self::SystemData) {
        for (_c, transform) in (&read_coin, &mut write_transform).join() {
            transform.rotate(0, -1, SPIN_SPEED * (input.frame_duration as Field));
            transform.rotate(2, -1, 0.345 * SPIN_SPEED * (input.frame_duration as Field));
        }
    }
}

pub struct PlayerCoinCollisionSystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for PlayerCoinCollisionSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadExpect<'a, Player>,
        ReadStorage<'a, Transform<V, V::M>>,
        ReadStorage<'a, Coin>,
        ReadStorage<'a, InPlayerCell>,
        ReadStorage<'a, Shape<V>>,
        Entities<'a>,
        Write<'a, DeletedEntities>,
        Write<'a, CoinsCollected>,
    );

    fn run(
        &mut self,
        (
			player,
			transform,
			coin,
			in_cell,
			shapes,
			entities,
			mut deleted,
			mut coins_collect
		) : Self::SystemData,
    ) {
        let pos = transform.get(player.0).unwrap().pos;

        for (_, _, shape, e) in (&coin, &in_cell, &shapes, &entities).join() {
            //collect the coin if close enough
            if shape.point_signed_distance(pos) < 0.1 {
                coins_collect.0 += 1;
                deleted.add(e);
            }
        }
    }
}
