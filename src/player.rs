use crate::geometry::Line;
use crate::components::*;
use crate::vector::{Field,VectorTrait};
use crate::camera::Camera;
use crate::collide::MoveNext;
use specs::prelude::*;
use specs::{Component,HashMapStorage};
use std::marker::PhantomData;
use crate::collide::BBox;


pub struct Player(pub Entity); //specifies entity of player

pub fn build_player<V : VectorTrait>(world : &mut World, camera : Camera<V>) {
	let player_entity = world.create_entity()
	    .with(BBox{min : V::ones()*(-0.1) + camera.pos, max : V::ones()*(0.1) + camera.pos})
	    .with(camera) //decompose
	    .with(MoveNext::<V>::default())
	    .with(MaybeTarget::<V>(None))
	    .build();

    world.insert(Player(player_entity));

}

const MAX_TARGET_DIST : Field = 10.;

pub struct ShapeTargetingSystem<V :VectorTrait>(pub PhantomData<V>);

impl<'a,V : VectorTrait> System<'a> for ShapeTargetingSystem<V> {
	type SystemData = (ReadExpect<'a,Player>,ReadStorage<'a,Camera<V>>,ReadStorage<'a,Shape<V>>,Entities<'a>,WriteStorage<'a,MaybeTarget<V>>);

	fn run(&mut self, (player, cameras, shapes, entities, mut targets) : Self::SystemData) {
		let camera = cameras.get(player.0).expect("Player has no camera");
		let target = shape_targeting(&camera,(&shapes,&*entities).join());
		*targets.get_mut(player.0).expect("Player has no target") = target;
		

	}
}
#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Cursor;

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct MaybeTarget<V : VectorTrait>(pub Option<Target<V>>);

pub struct Target<V : VectorTrait> {
	pub entity : Entity,
	pub distance : Field,
	pub point : V,

}

fn shape_targeting<'a, V : VectorTrait, I : std::iter::Iterator<Item=(&'a Shape<V>,Entity)>>(camera : &Camera<V>, iter : I) -> MaybeTarget<V> {
	let pos = camera.pos;
	let dir = camera.frame[-1];
	let ray = Line(pos, pos + dir*MAX_TARGET_DIST);

	//loop through all shapes and check for nearest intersection
	let mut closest : Option<(Entity,Field,V)> = None;
	for (shape,e) in iter {
		for intersect_point in shape.line_intersect(&ray,true) { //find intersections of ray with visible faces
			let distsq = (intersect_point - pos).norm_sq();
			closest = match closest {
				Some((_cle, cldistsq, _clpoint)) => match distsq > cldistsq {
					true => Some((e,distsq,intersect_point)),
					false => closest,
				}
				None => Some((e,distsq,intersect_point)),
			}
		} 
	}
	match closest {
		Some((e,distsq,point)) => MaybeTarget(Some(Target{entity : e, distance : distsq.sqrt(), point})),
		None => MaybeTarget(None),
	}
}