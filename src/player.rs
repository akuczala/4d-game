use crate::vector::VectorTrait;
use crate::camera::Camera;
use crate::collide::MoveNext;
use specs::prelude::*;

use crate::collide::BBox;

pub struct Player(pub Entity); //specifies entity of player

pub fn build_player<V : VectorTrait>(world : &mut World, camera : Camera<V>) {
	let player_entity = world.create_entity()
	    .with(BBox{min : V::ones()*(-0.1) + camera.pos, max : V::ones()*(0.1) + camera.pos})
	    .with(camera) //decompose
	    .with(MoveNext::<V>::default())
	    .build();

    world.insert(Player(player_entity));
}
