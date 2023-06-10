use crate::cleanup::DeletedEntities;
use crate::vector::{VectorTrait,Field};
use crate::components::*;
use core::marker::PhantomData;
use crate::input::Input;
use specs::prelude::*;
use specs::{Component,VecStorage};
use crate::geometry::shape::RefShapes;

#[derive(Default,Debug)]
pub struct CoinsCollected(pub u32);

#[derive(Component)]
#[storage(VecStorage)]
pub struct Coin;

const SPIN_SPEED : Field = 2.0;

pub struct CoinSpinningSystem<V : VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for CoinSpinningSystem<V> {

    type SystemData = (
		ReadStorage<'a,Coin>,
		ReadExpect<'a,Input>,
		ReadExpect<'a,RefShapes<V>>,
		ReadStorage<'a,ShapeLabel>,
		WriteStorage<'a,Shape<V>>,
		WriteStorage<'a,Transform<V>>
	);

    fn run(&mut self, (read_coin, input, ref_shape, read_shape_label, mut write_shape, mut write_transform) : Self::SystemData) {

        for (_c, shape_label, shape, transform)
		in (&read_coin, &read_shape_label, &mut write_shape, &mut write_transform).join() {
			*transform = transform.with_rotation(0,-1,SPIN_SPEED*(input.frame_duration as Field));
			*transform = transform.with_rotation(2,-1,0.345*SPIN_SPEED*(input.frame_duration as Field));
			shape.update_from_ref(ref_shape.get(shape_label).expect("Coin shape not found"), transform);
        }

    }
}

pub struct PlayerCoinCollisionSystem<V : VectorTrait>(pub PhantomData<V>);
impl<'a, V : VectorTrait> System<'a> for PlayerCoinCollisionSystem<V> {
	type SystemData = (
		ReadExpect<'a,Player>,
		ReadStorage<'a,Transform<V>>,
		ReadStorage<'a,Coin>,
		ReadStorage<'a,InPlayerCell>,
		ReadStorage<'a,Shape<V>>,Entities<'a>,
		Write<'a,DeletedEntities>,
		Write<'a, CoinsCollected>
	);

	fn run(&mut self, (player, transform, coin, in_cell, shapes, entities, mut deleted, mut coins_collect) : Self::SystemData) {
		let pos = transform.get(player.0).unwrap().pos;

		for (_, _, shape, e) in (&coin, &in_cell, &shapes, &entities).join() {
			//collect the coin if close enough
			if shape.point_signed_distance(pos) < 0.1 {
				coins_collect.0 += 1;
				deleted.add(e);
				entities.delete(e).unwrap();
			}

		}


	}
}