use crate::input::Input;
use crate::player::Player;
use crate::spatial_hash::{SpatialHashSet,HashInt};
use crate::geometry::Shape;
use crate::vector::{VectorTrait,Field};
use crate::components::{ShapeType,Convex,Transform,Transformable,Camera};
use specs::prelude::*;
use specs::{Component};
use std::marker::PhantomData;
use itertools::Itertools;

pub const PLAYER_COLLIDE_DISTANCE: Field = 0.2;

#[derive(Component)]
#[storage(VecStorage)]
pub struct StaticCollider;

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct InPlayerCell;

//axis-aligned bounding box
#[derive(Component)]
#[storage(VecStorage)]
pub struct BBox<V : VectorTrait> {
	pub min : V,
	pub max : V,
}

pub trait HasBBox<V : VectorTrait>: specs::Component {
	fn calc_bbox(&self) -> BBox<V>;
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct MoveNext<V : VectorTrait>{pub next_dpos : Option<V>, pub can_move : Option<bool>}
impl<V : VectorTrait> Default for MoveNext<V> {
	fn default() -> Self {
		Self{next_dpos : None, can_move : None}
	}
}
impl<V: VectorTrait> Transformable<V> for MoveNext<V> {
	fn set_identity(mut self) -> Self {
		self.next_dpos = Some(V::zero());
		self
	}
	fn transform(&mut self, transform: Transform<V>) {
		self.next_dpos = match self.next_dpos {
			Some(v) => Some(v + transform.pos),
			None => Some(transform.pos)
		};
	}
}
//print entities in the same cell as the player's bbox
pub struct MovePlayerSystem<V>(pub PhantomData<V>);

impl<'a, V : VectorTrait> System<'a> for MovePlayerSystem<V> {

	type SystemData = (
		ReadExpect<'a,Player>,
		WriteStorage<'a,MoveNext<V>>,
		WriteStorage<'a,Transform<V>>,
		WriteStorage<'a,Camera<V>>,
	);

	fn run(&mut self, (player, mut write_move_next, mut transforms, mut cameras) : Self::SystemData) {
		let move_next = write_move_next.get_mut(player.0).unwrap();
		let transform = transforms.get_mut(player.0).unwrap();
		let camera = cameras.get_mut(player.0).unwrap();
		match move_next {
			MoveNext{next_dpos : Some(next_dpos), can_move : Some(true)} => {
				*transform = transform.with_translation(*next_dpos);
				camera.update(transform);
			},
			_ => (),
		};
		*move_next = MoveNext::default(); //clear movement

	}
}

//this system is not used yet
pub struct UpdateBBoxSystem<V: VectorTrait,T: HasBBox<V>>(pub PhantomData<V>, pub PhantomData<T>);

impl<'a,V: VectorTrait,T: HasBBox<V>> System<'a> for UpdateBBoxSystem<V,T> {

	type SystemData = (ReadStorage<'a,T>,WriteStorage<'a,BBox<V>>);

	fn run(&mut self, (read_shape, mut write_bbox) : Self::SystemData) {
		for (shape, bbox) in (&read_shape, &mut write_bbox).join() {
			*bbox = shape.calc_bbox();
		}
	}
}

impl<V: VectorTrait> HasBBox<V> for Shape<V> {
	fn calc_bbox(&self) -> BBox<V> {
		let verts = &self.verts;

		//take smallest and largest components to get bounding box
		let (mut min, mut max) = (verts[0],verts[0]);
		for &v in verts.iter() {
			min = min.zip_map(v,Field::min); 
			max = max.zip_map(v,Field::max);
		}
		BBox{min,max}
	}
}


pub fn create_spatial_hash<V : VectorTrait>(world : &mut World) {
	//add bbox entities and initialize spatial hash set
    let (mut max, mut min) = (V::zero(), V::zero());
    let mut max_lengths = V::zero();
    for bbox in (&world.read_component::<BBox<V>>()).join() {
        min = min.zip_map(bbox.min,Field::min); 
        max = max.zip_map(bbox.max,Field::max);
        max_lengths = max_lengths.zip_map(bbox.max - bbox.min,Field::max);
    }
    //println!("Min/max: {},{}",min,max);
    //println!("Longest sides {}",max_lengths);
    world.insert(
        SpatialHashSet::<V,Entity>::new(
            min*1.5, //make bounds slightly larger than farthest points
            max*1.5,
            max_lengths*1.1 //make cell size slightly larger than largest bbox dimensions
        )
    );
    //enter bboxes into hash set
    BBoxHashingSystem(PhantomData::<V>).run_now(&world);
}

//enter each statically colliding entity into every cell containing its bbox volume (either 1, 2, 4 ... up to 2^d cells)
//assuming that cells are large enough that all bboxes can fit in a cell
//for static objects, it is cheap to hash the volume since we need only do it once
pub struct BBoxHashingSystem<V>(pub PhantomData<V>);

impl<'a,V : VectorTrait> System<'a> for BBoxHashingSystem<V> {

	type SystemData = (ReadStorage<'a,BBox<V>>,Entities<'a>,WriteExpect<'a,SpatialHashSet<V,Entity>>);

	fn run(&mut self, (read_bbox, entities, mut write_hash) : Self::SystemData) {
		let hash = &mut write_hash;
		for (bbox, entity) in (&read_bbox,&*entities).join() {
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
	type SystemData = (ReadExpect<'a,Player>,WriteStorage<'a,BBox<V>>,ReadStorage<'a,Transform<V>>);

	fn run(&mut self, (player, mut write_bbox, transform) : Self::SystemData) {
		let pos = transform.get(player.0).unwrap().pos;
		let mut bbox = write_bbox.get_mut(player.0).unwrap();
		bbox.min = pos - V::constant(0.2);
		bbox.max = pos + V::constant(-0.2);
	}
}
//print entities in the same cell as the player's bbox
pub struct CollisionTestSystem<V>(pub PhantomData<V>);

impl<'a, V : VectorTrait> System<'a> for CollisionTestSystem<V> {

	type SystemData = (
		ReadExpect<'a,Input>,
		ReadExpect<'a,Player>,
		ReadStorage<'a,Transform<V>>,
		ReadStorage<'a,Shape<V>>,
		ReadStorage<'a,ShapeType<V>>,
		ReadStorage<'a,BBox<V>>,
		ReadExpect<'a,SpatialHashSet<V,Entity>>
	);

	fn run(&mut self, (input, player, transform, shapes, shape_types, bbox, hash) : Self::SystemData) {
		use glium::glutin::event::VirtualKeyCode as VKC;
		if input.helper.key_released(VKC::Space) {
			//let mut out_string = "Entities: ".to_string();
			let entities_in_bbox = get_entities_in_bbox(&bbox.get(player.0).unwrap(),&hash);
			let player_pos = transform.get(player.0).unwrap().pos;
			if entities_in_bbox.iter().any(
				|&e| match shape_types.get(e).unwrap() {
					ShapeType::Convex(_convex) => Convex::point_within(player_pos,0.1, &shapes.get(e).unwrap().faces),
					_ => false,
				}
			) {
				println!("in thing")
			} else {
				println!("not in thing")
			}
			// for e in entities_in_bbox {
			// 	out_string = format!("{} {},", out_string, e.id())
			// }
			// println!("{}", out_string);
		}

	}
}
//stop movement through entites indexed in spatial hash set
//need only run these systems when the player is moving
pub struct PlayerCollisionDetectionSystem<V>(pub PhantomData<V>);

impl<'a, V : VectorTrait> System<'a> for PlayerCollisionDetectionSystem<V> {

	type SystemData = (ReadExpect<'a,Player>,ReadStorage<'a,BBox<V>>,WriteStorage<'a,InPlayerCell>,ReadExpect<'a,SpatialHashSet<V,Entity>>);

	fn run(&mut self, (player, bbox, mut in_cell, hash) : Self::SystemData) {
		in_cell.clear(); //clear previously marked
		//maybe we should use the anticipated player bbox here
		let entities_in_bbox = get_entities_in_bbox(&bbox.get(player.0).unwrap(),&hash);
		for &e in entities_in_bbox.iter() {
			in_cell.insert(e, InPlayerCell).expect("PlayerCollisionDetectionSystem: entity in spatial hash doesn't exist");
		}
	}
}

pub struct PlayerStaticCollisionSystem<V :VectorTrait>(pub PhantomData<V>);
impl<'a, V : VectorTrait> System<'a> for PlayerStaticCollisionSystem<V> {

	type SystemData = (
		ReadExpect<'a,Player>,
		ReadStorage<'a,Transform<V>>,
		WriteStorage<'a,MoveNext<V>>,
		ReadStorage<'a,Shape<V>>,
		ReadStorage<'a,ShapeType<V>>,
		ReadStorage<'a,StaticCollider>,
		ReadStorage<'a,InPlayerCell>
	);

	fn run(&mut self, (player, transform, mut write_move_next, shape, shape_types, static_collider, in_cell) : Self::SystemData) {
		let move_next = write_move_next.get_mut(player.0).unwrap();
		match move_next {
			MoveNext{next_dpos : Some(_next_dpos), can_move : Some(true)} => {
				let pos = transform.get(player.0).unwrap().pos;
				for (shape, shape_type, _, _) in (&shape, &shape_types, &static_collider, &in_cell).join() {

					let next_dpos = move_next.next_dpos.unwrap();
					let (normal, dist) = shape.point_normal_distance(pos);

					if (dist < PLAYER_COLLIDE_DISTANCE) & match shape_type {
							ShapeType::SingleFace(single_face) => single_face.subface_normal_distance(pos).1 < PLAYER_COLLIDE_DISTANCE,
							ShapeType::Convex(_) => true }
					{
						//push player away along normal of nearest face (projects out -normal)
						//but i use abs here to guarantee the face always repels the player
						let new_dpos = next_dpos + (normal)*(normal.dot(next_dpos).abs());

						move_next.next_dpos = Some(new_dpos);
						//println!("{}",normal);
					}
				}
				
			},
			_ => (),
		}
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
