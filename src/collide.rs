use crate::spatial_hash::SpatialHashSet;
use crate::geometry::Shape;
use crate::vector::{VectorTrait,Field};
use specs::prelude::*;
use specs::{Component};
use std::marker::PhantomData;

//use itertools::Itertools;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Collider;

//axis-aligned bounding box
#[derive(Component)]
#[storage(VecStorage)]
pub struct BBox<V : VectorTrait> {
	pub min : V,
	pub max : V,
}

pub struct UpdateBBoxSystem<V>(pub PhantomData<V>);

impl<'a,V : VectorTrait> System<'a> for UpdateBBoxSystem<V> {

	type SystemData = (ReadStorage<'a,Shape<V>>,WriteStorage<'a,BBox<V>>);

	fn run(&mut self, (read_shape, mut write_bbox) : Self::SystemData) {
		for (shape, bbox) in (&read_shape, &mut write_bbox).join() {
			*bbox = calc_bbox(shape);
		}
	}
}

pub fn calc_bbox<V : VectorTrait>(shape : &Shape<V>) -> BBox<V> {
	let verts = &shape.verts;

	//take smallest and largest components to get bounding box
	let (mut min, mut max) = (verts[0],verts[0]);
	for &v in verts.iter() {
		min = min.zip_map(v,Field::min); 
		max = max.zip_map(v,Field::max);
	}
	BBox{min,max}
}

// pub enum BBoxHashResult {

// }

pub struct BBoxHashingSystem<V>(pub PhantomData<V>);

impl<'a,V : VectorTrait> System<'a> for BBoxHashingSystem<V> {

	type SystemData = (ReadStorage<'a,Collider>,ReadStorage<'a,BBox<V>>,Entities<'a>,WriteExpect<'a,SpatialHashSet<V,Entity>>);

	fn run(&mut self, (read_collider, read_bbox, entities, mut write_hash) : Self::SystemData) {
		let hash = &mut write_hash;
		for (_, bbox, entity) in (&read_collider,&read_bbox,&*entities).join() {
			hash.insert(&bbox.min, entity); hash.insert(&bbox.max, entity);

		}
	}
}

pub struct CollisionTestSystem<V>(pub PhantomData<V>);

// impl<'a, V : VectorTrait> System<'a> for CollisionTestSystem<V> {

// 	type SystemData = (ReadExpect<'a,Camera<V>>,ReadExpect<'a,SpatialHashSet<V,Entity>>);

// 	fn run(&mut self, (read_camera, read_hash)) {
// 		let hash = &read_hash;
// 		let camera = &read_camera;


// 	}
// }

//for simplicity, let's assume that every colliding entity has a bbox small enough that it can occupy 1, 2 or at most 4 cells.
//this means that we should have the cell sizes larger than the longest object, for each axis
//this may be problematic if we want to consider entities with size comparable to the scene size - then the hash map is useless

fn get_entities_in_player_bbox<V : VectorTrait>(collider_bbox : BBox<V>, hash : &SpatialHashSet<V,Entity>) {

}
