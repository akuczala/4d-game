use crate::geometry::Shape;
use crate::vector::{VectorTrait,Field};
use specs::prelude::*;
use specs::{Component};
use std::marker::PhantomData;

use itertools::Itertools;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Collider;

//axis-aligned bounding box
#[derive(Component)]
#[storage(VecStorage)]
pub struct BBox<V : VectorTrait> {
	min : V,
	max : V,
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

use std::collections::{HashMap,HashSet};
use std::hash::Hash;

type HashInt = u32;

//NOTE: V here is "VALUE" not vector
pub struct SpatialHash<K : VectorTrait, V> {
	map : HashMap<HashInt, V>,
	min : K,
	max : K,
	length : K,
	//cell_size : K, //not strictly necessary
	n_cells : Vec<HashInt>, //size K::DIM
	multiplier : Vec<HashInt> //size K::DIM
}
impl<K : VectorTrait, V> SpatialHash<K, V> {

	//desired_cell_size is only a suggestion. actual cell size will divide (max-min)
	fn new(min : K, max : K, desired_cell_size : K) -> Self {
		let length = max - min;
		let n_cells : Vec<HashInt> = length.zip_map(desired_cell_size,|l,s| l/s).iter().map(|&f| f as HashInt).collect();

		let mut multiplier : Vec<HashInt> = vec![1];
		for &n in n_cells.iter() {
			//some tomfoolery here required to satisfy the borrow checker
			let last = {let last = multiplier.last(); match last {Some(&n) => n, None => 1}};
			multiplier.push(last*n);
		}
		//unsure how to convert arr to VectorTrait generically
		// let cell_size = length
		// 	.zip_map(
		// 		VectorTrait::from_arr(&n_cells.iter().map(|&u| u as Field).collect()),
		// 		|l,n| l/n
		// 	);
		Self{map : HashMap::new(), min, max, length, n_cells, multiplier}
	}
	//hash is sum_i floor((p[i]-min[i])/len([i])*mult[i]
	//should have a check for outside of hash region
	fn hash(&self, &point : &K) -> HashInt {
		(point - self.min).zip_map(self.length,|p,l| p/l).iter()
		.zip(self.n_cells.iter()).zip(self.multiplier.iter())
		.map(|((f,&n),&m)| ((f*(n as Field)) as HashInt)*m)
		.sum()

	}
	fn get(&self, point : &K) -> Option<&V> {
		self.get_from_cell(self.hash(point))
	}
	fn get_mut(&mut self, point : &K) -> Option<&mut V> {
		self.get_mut_from_cell(self.hash(point))
	}

	fn get_from_cell(&self, cell : HashInt) -> Option<&V> {
		self.map.get(&cell)
	}
	fn get_mut_from_cell(&mut self, cell : HashInt) -> Option<&mut V> {
		self.map.get_mut(&cell)
	}
	fn insert(&mut self, point : &K, value : V) -> Option<V> {
		self.map.insert(self.hash(point),value)
	}
	fn remove(&mut self, point: &K) -> Option<V> {
		self.map.remove(&self.hash(point))
	}
}
impl<K : VectorTrait, V : std::fmt::Display> std::fmt::Display for SpatialHash<K, V> {
	// This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    	let mut out : String = "".to_owned();
    	for (key, val) in self.map.iter() {
    		out = format!("{} \n key: {} val: {}",out, key, val);
		}
        write!(f, "{}",out)
	}
}

//in the case where each key is a set of T, we can cumulatively add, remove elements
pub struct SpatialHashSet<K : VectorTrait, T>(pub SpatialHash<K, HashSet<T>>)
	where T : Eq + Hash; //must have these traits to be put in hash set

impl <K : VectorTrait, T> SpatialHashSet<K, T>
	where T : Eq + Hash {
	fn hash(&self, point : &K) -> HashInt {
		self.0.hash(point)
	}
	fn get(&self, point : &K) -> Option<&HashSet<T>> {
		self.0.get(point)
	}
	fn get_from_cell(&self, cell : HashInt) -> Option<&HashSet<T>> {
		self.0.get_from_cell(cell)
	}
	//create new set in bin or append to existing set
	fn insert(&mut self, point : &K, item : T) {
		let maybe_set = self.0.get_mut(point);
		match maybe_set {
			Some(set) => {set.insert(item);},
			None => {
				let mut new_set = HashSet::new();
				new_set.insert(item);
				self.0.insert(&point,new_set);
			},
		};

	}
	fn remove(&mut self, point : &K, item : &T) -> bool {
		let maybe_set = self.0.get_mut(point);
		match maybe_set {
			Some(set) => set.remove(item),
			None => false,
		}
	}
	fn clear_cell(&mut self, point : &K) -> Option<HashSet<T>> {
		self.0.remove(&point)
	}
}
// trait CheapTrick: std::fmt::Display {}
// impl<T : std::fmt::Display> CheapTrick for HashSet<T> {
// 	// This trait requires `fmt` with this exact signature.
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//     	let mut out : String = "".to_owned();
//     	for item in self.iter() {
//     		out = format!("{}, {}",out, item);
// 		}
//         write!(f, "{}", out)
// 	}
// }


#[test]
fn test_hash() {
	use crate::vector::Vec2;
	let hash = SpatialHash::<Vec2, u32>::new(
		Vec2::new(-0.01,-0.01),
		Vec2::new(10.01, 10.01),
		Vec2::new(1., 1.)
	);
	for (i,x) in (0..9).map(|i| (i as Field) + 0.9).enumerate() {
		for (j,y) in (0..9).map(|i| (i as Field) + 0.5).enumerate() {
		let testvec = Vec2::new(x,y);
		//hash.insert();
		let hashval = hash.hash(&testvec);
		//println!("{:?}",hashval);
		assert_eq!(hashval,(i as HashInt) + 10*(j as HashInt))
		}
	}
		
}
#[test]
fn test_hash2() {
	use crate::vector::Vec3;
	type V = Vec3;
	let mut hash = SpatialHash::<V, u32>::new(
		V::new(0.,0.,0.),
		V::new(10., 10.,10.),
		V::new(3., 3., 3.)
	);
	hash.insert(&V::new(0.5,1.5,7.), 5);
	assert_eq!(hash.insert(&V::new(0.5,1.5,8.), 8),Some(5 as HashInt));
	hash.insert(&V::new(9.5,5.,0.5), 1);

	//println!("dum dum {}",hash);
	assert_eq!(hash.get(&V::new(0.7,1.1,9.)),Some(&(8 as HashInt)));
	assert_eq!(hash.get(&V::new(8.,6.,0.5)),Some(&(1 as HashInt)));

	assert_eq!(hash.remove(&V::new(7.,4.,0.)), Some(1));

	assert_eq!(hash.get(&V::new(8.,6.,0.5)),None);
}