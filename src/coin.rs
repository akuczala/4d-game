use crate::player::Player;
use crate::camera::Camera;
use crate::collide::InPlayerCell;
use crate::cleanup::DeletedEntities;
use crate::vector::{VectorTrait,Field};
use core::marker::PhantomData;
use crate::geometry::{Shape};
use crate::input::Input;
use specs::prelude::*;
use specs::{Component,VecStorage};

#[derive(Default,Debug)]
pub struct CoinsCollected(pub u32);

#[derive(Component)]
#[storage(VecStorage)]
pub struct Coin;

const SPIN_SPEED : Field = 2.0;

pub struct CoinSpinningSystem<V : VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for CoinSpinningSystem<V> {

    type SystemData = (ReadStorage<'a,Coin>,ReadExpect<'a,Input>,WriteStorage<'a,Shape<V>>);

    fn run(&mut self, (read_coin, input, mut write_shape) : Self::SystemData) {

        for (_c, shape) in (&read_coin, &mut write_shape).join() {
        	shape.rotate(0,-1,SPIN_SPEED*(input.frame_duration as Field));
        	shape.rotate(2,-1,0.345*SPIN_SPEED*(input.frame_duration as Field));
        }

    }
}

pub struct PlayerCoinCollisionSystem<V : VectorTrait>(pub PhantomData<V>);
impl<'a, V : VectorTrait> System<'a> for PlayerCoinCollisionSystem<V> {
	type SystemData = (ReadExpect<'a,Player>, ReadStorage<'a,Camera<V>>,
		ReadStorage<'a,Coin>,ReadStorage<'a,InPlayerCell>, ReadStorage<'a,Shape<V>>,Entities<'a>,Write<'a,DeletedEntities>, Write<'a, CoinsCollected>);

	fn run(&mut self, (player, camera, coin, in_cell, shapes, entities, mut deleted, mut coins_collect) : Self::SystemData) {
		let pos = camera.get(player.0).unwrap().pos;

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