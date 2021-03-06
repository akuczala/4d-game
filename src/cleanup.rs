
use crate::clipping::ShapeClipState;
use crate::spatial_hash::SpatialHashSet;
use specs::prelude::*;

use crate::vector::VectorTrait;

use core::marker::PhantomData;

pub struct DeletedEntities(pub Vec<Entity>);
impl Default for DeletedEntities {
	fn default() -> Self {
		Self(vec![])
	}
}
impl DeletedEntities {
	pub fn add(&mut self, e : Entity) {
		self.0.push(e);
	}
}

pub struct ShapeCleanupSystem<V : VectorTrait>(pub PhantomData<V>);
impl<'a, V : VectorTrait> System<'a> for ShapeCleanupSystem<V> {
	type SystemData = (Write<'a,DeletedEntities>,WriteStorage<'a,ShapeClipState<V>>,WriteExpect<'a,SpatialHashSet<V,Entity>>);

	fn run(&mut self, (mut deleted, mut shape_clip, mut hash) : Self::SystemData) {
		let len = deleted.0.len();
		for e in deleted.0.drain(0..len) {

			//this is a little slow since we need to check all cells, but we don't expect this to occur often
			hash.remove_from_all(&e); //remove from spatial hash

			for clip in (&mut shape_clip).join() {
				clip.remove(&e);
				
			}
			
		}
	}
}