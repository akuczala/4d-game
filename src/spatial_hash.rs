use crate::geometry::Shape;
use crate::collide::{UpdateBBoxSystem,BBoxHashingSystem,BBox,calc_bbox};
use std::marker::PhantomData;
use std::collections::{HashMap,HashSet};
use std::hash::Hash;
use crate::vector::{VectorTrait,Field};

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

	pub fn new(min : K, max : K, desired_cell_size : K) -> Self {
		Self(SpatialHash::new(min,max,desired_cell_size))
	}
	pub fn hash(&self, point : &K) -> HashInt {
		self.0.hash(point)
	}
	pub fn get(&self, point : &K) -> Option<&HashSet<T>> {
		self.0.get(point)
	}
	pub fn get_from_cell(&self, cell : HashInt) -> Option<&HashSet<T>> {
		self.0.get_from_cell(cell)
	}
	//create new set in bin or append to existing set
	pub fn insert(&mut self, point : &K, item : T) {
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
	pub fn remove(&mut self, point : &K, item : &T) -> bool {
		let maybe_set = self.0.get_mut(point);
		match maybe_set {
			Some(set) => set.remove(item),
			None => false,
		}
	}
	pub fn clear_cell(&mut self, point : &K) -> Option<HashSet<T>> {
		self.0.remove(&point)
	}
	
}
use specs::Entity;
impl<K : VectorTrait> SpatialHashSet<K, Entity> {
	fn print(&self) {
    	let mut out : String = "".to_owned();
    	for (key, val) in self.0.map.iter() {
    		out = format!("{} \n key: {} val: {}",out, key, hashset_string(val));
		}
        println!("{}",out)
	}
}
// impl<K : VectorTrait, T : std::fmt::Display> std::fmt::Display for SpatialHashSet<K, T>
// where T : Eq + Hash {
// 	// This trait requires `fmt` with this exact signature.
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//     	let mut out : String = "".to_owned();
//     	for (key, val) in self.0.map.iter() {
//     		out = format!("{} \n key: {} val: {}",out, key, hashset_string(val));
// 		}
//         write!(f, "{}",out)
// 	}
// }
// trait CheapTrick: std::fmt::Display {}
// impl<T : std::fmt::Display> CheapTrick for SpatialHashSet<T> {
// 	// This trait requires `fmt` with this exact signature.
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//     	let mut out : String = "".to_owned();
//     	for item in self.iter() {
//     		out = format!("{}, {}",out, item);
// 		}
//         write!(f, "{}", out)
// 	}
// }
fn entity_string(entity : &Entity) -> String {
	format!("{}",entity.id())
}
fn hashset_string(hash : &HashSet<Entity>) -> String {
	let mut out : String = "".to_owned();
	for item in hash.iter() {
		out = format!("{} {},",out, entity_string(item));
	}
    format!("{}", out)
}


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

#[test]
fn test_shape_hash() {
	use specs::prelude::*;
	use crate::vector::Vec3;
	use crate::collide::Collider;
	type V = Vec3;

	let mut world = World::new();
	world.register::<Shape<V>>();
	world.register::<Collider>();
	world.register::<BBox<V>>();

    let shapes = crate::build_level::build_shapes_3d();

    //let shapes_len = shapes.len();
    //let coin_shape = shapes.pop();
    let (mut max, mut min) = (V::zero(), V::zero());
    let mut max_lengths = V::zero();
    for shape in shapes.into_iter() {
    	let bbox = calc_bbox(&shape);
    	min = min.zip_map(bbox.min,Field::min); 
		max = max.zip_map(bbox.max,Field::max);
		max_lengths = max_lengths.zip_map(bbox.max - bbox.min,Field::max);
        world.create_entity()
        .with(bbox)
        .with(shape)
        .with(Collider)
        .build();
    }
    println!("Min/max: {},{}",min,max);
    println!("Longest sides {}",max_lengths);
    world.insert(
		SpatialHashSet::<Vec3,Entity>::new(
			min*1.5, //make bounds slightly larger than farthest points
			max*1.5,
			max_lengths*1.1 //make cell size slightly larger than largest shape dimensions
		)
	);


    let mut dispatcher = DispatcherBuilder::new()
    	.with(UpdateBBoxSystem(PhantomData::<V>),"bbox",&[])
    	.with(BBoxHashingSystem(PhantomData::<V>),"hash",&["bbox"])
    	.build();

    dispatcher.dispatch(&mut world);

    let hash = world.read_resource::<SpatialHashSet<V, Entity>>();
    hash.print();
    for n in hash.0.n_cells.iter() {
    	println!("cells: {}",n);
    }


}