use crate::geometry::Shape;
use crate::vector::{VectorTrait,Field};
use specs::prelude::*;
use specs::{Component};
use std::marker::PhantomData;

#[derive(Component)]
#[storage(VecStorage)]
struct Collider;

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

use std::collections::HashMap;
type HashInt = u32;
pub struct SpatialHash<V : VectorTrait, K> {
	map : HashMap<HashInt, K>,
	min : V,
	max : V,
	cell_size : V,
}
impl<K : VectorTrait, V> SpatialHash<K, V> {
	fn hash(&self, &point : &K) -> HashInt {
		let lengths = self.max - self.min;
		let float_stuff = lengths.zip_map(self.cell_size,|l,s| l/s).zip_map(point,|f,p| f*p).fold(None,|a, b| a + b);
		float_stuff as HashInt

	}
	fn get(&self, point : &K) -> Option<&V> {
		self.map.get(&self.hash(point))

	}
	fn insert(&mut self, point : &K, value : V) -> Option<V> {
		self.map.insert(self.hash(point),value)
	}
}

#[test]
fn test_hash() {
	use crate::vector::Vec2;
	let hash = SpatialHash::<Vec2, u32>{
		map : HashMap::new(),
		min : Vec2::new(0.,0.),
		max : Vec2::new(10., 10.),
		cell_size : Vec2::new(1., 1.),
	};
	let testvec = Vec2::new(1.5,1.5);
	//hash.insert();
	let hashval = hash.hash(&testvec);
	println!("{:?}",hashval);
	assert!(hashval==0)
}