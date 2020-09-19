use crate::input::Input;
use crate::camera::Camera;
use crate::engine::Player;
use crate::spatial_hash::{SpatialHashSet,HashInt};
use crate::geometry::Shape;
use crate::vector::{VectorTrait,Field,Translatable};
use specs::prelude::*;
use specs::{Component};
use std::marker::PhantomData;
use itertools::Itertools;
//use itertools::Itertools;

#[derive(Component)]
#[storage(VecStorage)]
pub struct StaticCollider;

//axis-aligned bounding box
#[derive(Component)]
#[storage(VecStorage)]
pub struct BBox<V : VectorTrait> {
	pub min : V,
	pub max : V,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct MoveNext<V : VectorTrait>{pub next_dpos : Option<V>, pub can_move : Option<bool>}
impl<V : VectorTrait> Default for MoveNext<V> {
	fn default() -> Self {
		Self{next_dpos : None, can_move : None}
	}
}
//print entities in the same cell as the player's bbox
pub struct MovePlayerSystem<V>(pub PhantomData<V>);

impl<'a, V : VectorTrait> System<'a> for MovePlayerSystem<V> {

	type SystemData = (ReadExpect<'a,Player>, WriteStorage<'a,MoveNext<V>>, WriteStorage<'a,Camera<V>>);

	fn run(&mut self, (player, mut write_move_next, mut camera) : Self::SystemData) {
		let move_next = write_move_next.get_mut(player.0).unwrap(); 
		match move_next {
			MoveNext{next_dpos : Some(next_dpos), can_move : Some(true)} => {
				camera.get_mut(player.0).unwrap().translate(*next_dpos);
			},
			_ => (),
		};
		*move_next = MoveNext::default(); //clear movement

	}
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

//enter each statically colliding entity into every cell containing its bbox volume (either 1, 2, 4 ... up to 2^d cells)
//assuming that cells are large enough that all bboxes can fit in a cell
//for static objects, it is cheap to hash the volume since we need only do it once
pub struct BBoxHashingSystem<V>(pub PhantomData<V>);

impl<'a,V : VectorTrait> System<'a> for BBoxHashingSystem<V> {

	type SystemData = (ReadStorage<'a,StaticCollider>,ReadStorage<'a,BBox<V>>,Entities<'a>,WriteExpect<'a,SpatialHashSet<V,Entity>>);

	fn run(&mut self, (read_collider, read_bbox, entities, mut write_hash) : Self::SystemData) {
		let hash = &mut write_hash;
		for (_, bbox, entity) in (&read_collider,&read_bbox,&*entities).join() {
			for cell in get_bbox_cells(&bbox,hash).into_iter() {
				hash.insert_at_cell(cell,entity);
			}

		}
	}
}
fn get_bbox_cells<V : VectorTrait>(bbox : &BBox<V>, hash : &SpatialHashSet<V,Entity>) -> Vec<HashInt> {
	let max_coords = hash.get_cell_coords(&bbox.max);
	let min_coords = hash.get_cell_coords(&bbox.min);
	let dcoords : Vec<HashInt> = max_coords.iter().zip(min_coords).map(|(max,min)| max - min).collect();
	get_dcoords_dcells(&dcoords,&hash.0.multiplier).into_iter()
		.map(|dc| hash.hash(&bbox.min) + dc) //add min (base) cell
		.collect()
}
//add an update_bbox marker
pub struct UpdatePlayerBBox<V>(pub PhantomData<V>);

impl<'a, V : VectorTrait> System<'a> for UpdatePlayerBBox<V> {
	type SystemData = (ReadExpect<'a,Player>,WriteStorage<'a,BBox<V>>,ReadStorage<'a,Camera<V>>);

	fn run(&mut self, (player, mut write_bbox, camera) : Self::SystemData) {
		let pos = camera.get(player.0).unwrap().pos;
		let mut bbox = write_bbox.get_mut(player.0).unwrap();
		bbox.min = pos - V::constant(0.2);
		bbox.max = pos + V::constant(-0.2);
	}
}
//print entities in the same cell as the player's bbox
pub struct CollisionTestSystem<V>(pub PhantomData<V>);

impl<'a, V : VectorTrait> System<'a> for CollisionTestSystem<V> {

	type SystemData = (ReadExpect<'a,Input>,ReadExpect<'a,Player>,ReadStorage<'a,Camera<V>>,ReadStorage<'a,Shape<V>>,ReadStorage<'a,BBox<V>>,ReadExpect<'a,SpatialHashSet<V,Entity>>);

	fn run(&mut self, (input, player, camera, shape, bbox, hash) : Self::SystemData) {
		use glium::glutin::event::VirtualKeyCode as VKC;
		if input.helper.key_released(VKC::Space) {
			//let mut out_string = "Entities: ".to_string();
			let entities_in_bbox = get_entities_in_bbox(&bbox.get(player.0).unwrap(),&hash);
			let player_pos = camera.get(player.0).unwrap().pos;
			if entities_in_bbox.iter().any(|&e| shape.get(e).unwrap().point_within(player_pos,0.1)) {
				println!("in thing")
			} else {
				println!("not in thing")
			};
			// for e in entities_in_bbox {
			// 	out_string = format!("{} {},", out_string, e.id())
			// }
			// println!("{}", out_string);
		}

	}
}
//stop movement through entites indexed in spatial hash set
pub struct PlayerCollisionDetectionSystem<V>(pub PhantomData<V>);

impl<'a, V : VectorTrait> System<'a> for PlayerCollisionDetectionSystem<V> {

	type SystemData = (ReadExpect<'a,Player>, ReadStorage<'a,Camera<V>>, WriteStorage<'a,MoveNext<V>>,
		ReadStorage<'a,Shape<V>>,ReadStorage<'a,BBox<V>>,ReadExpect<'a,SpatialHashSet<V,Entity>>);

	fn run(&mut self, (player, camera, mut write_move_next, shape, bbox, hash) : Self::SystemData) {
		let move_next = write_move_next.get_mut(player.0).unwrap();
		match move_next {
			MoveNext{next_dpos : Some(_next_dpos), can_move : Some(true)} => {
				//maybe we should use the anticipated player bbox here
				let pos = camera.get(player.0).unwrap().pos;
				let entities_in_bbox = get_entities_in_bbox(&bbox.get(player.0).unwrap(),&hash);

				for &e in entities_in_bbox.iter() {
					let shape = shape.get(e).unwrap();
					let next_dpos = move_next.next_dpos.unwrap();
					let (normal, dist) = shape.point_normal_distance(pos);
					
					if dist < 0.2 {
						//push player away along normal of nearest face (projects out -normal)
						//but i use abs here to guarantee the face always repels the player
						let new_dpos = next_dpos + (normal)*(normal.dot(next_dpos).abs());

						move_next.next_dpos = Some(new_dpos);
						//println!("{}",normal);
					}
				}
				
			},
			_ => (),
		};
		

	}
}

//for simplicity, let's assume that every colliding entity has a bbox small enough that it can occupy up to 2^d cells.
//this means that we should have the cell sizes larger than the longest object, for each axis
//this may be problematic if we want to consider entities with size comparable to the scene size - then the hash map is useless

fn get_entities_in_bbox<V : VectorTrait>(bbox : &BBox<V>, hash : &SpatialHashSet<V,Entity>) -> Vec<Entity> {
	get_bbox_cells(bbox,hash).into_iter()
		.filter_map(|cell| hash.get_from_cell(cell)) //get hash set from each cell, if it exists
		.map(|hashset| hashset.iter()) //get iterator from each hash set
		.flatten()
		.map(|&entity| entity) //deref
		.unique() //remove duplicate entities
		.collect()

}
fn get_bits(n : HashInt, n_bits : HashInt) -> impl Iterator<Item=HashInt> { //might be more sensible to be bool?
	(0..n_bits).map(move |k| n.rotate_right(k)%2)
}
//could memoize results here if too slow. but should be fast
//assumes entries of dcoords are 0 or 1
fn get_dcoords_dcells(dcoords : &Vec<HashInt>, mult : &Vec<HashInt>) -> Vec<HashInt> {
		//let dim = dcoords.len();
		assert!(dcoords.iter().all(|&d| d == 0 || d== 1));
		let dpos : Vec<usize> = dcoords.iter().enumerate().filter_map(|(i,d)|match d {1 => Some(i), _ => None}).collect();

		let box_dim = dcoords.iter().sum();
		let kmax : HashInt = (2 as HashInt).pow(box_dim);
		let mut out_vec : Vec<HashInt> = vec![];

		for k in 0..kmax {
			let k_bits = get_bits(k,box_dim);
			let cell = k_bits.zip(dpos.iter()).map(|(kb,&d)| kb*mult[d]).sum::<HashInt>();
			//println!("CELL: {:?} vs {:?}",cell,k);
			out_vec.push(cell)
			
		}
		out_vec
	}

#[test]
fn dcoords_cells_test() {
	//let result : Vec<HashInt> = (0..8).collect();
	let mult = vec![1,2,4];
	//let dcoords = vec![1,1,1];
	
	
	assert_eq!(get_dcoords_dcells(&vec![1,1,1],&mult),{let r : Vec<HashInt> = (0..8).collect(); r});
	assert_eq!(get_dcoords_dcells(&vec![0,0,0],&mult),vec![0]);
	assert_eq!(get_dcoords_dcells(&vec![1,0,0],&mult),vec![0,1]);
	assert_eq!(get_dcoords_dcells(&vec![0,1,0],&mult),vec![0,2]);
	assert_eq!(get_dcoords_dcells(&vec![0,0,1],&mult),vec![0,4]);
	assert_eq!(get_dcoords_dcells(&vec![1,1,0],&mult),vec![0,1,2,3]);
	assert_eq!(get_dcoords_dcells(&vec![0,1,1],&mult),vec![0,2,4,6]);
	assert_eq!(get_dcoords_dcells(&vec![1,0,1],&mult),vec![0,1,4,5]);
}
