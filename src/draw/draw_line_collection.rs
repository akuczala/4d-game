use std::marker::PhantomData;

use specs::{ReadStorage, WriteExpect, System, Join};

use crate::{graphics::colors::Color, geometry::Line, vector::VectorTrait, ecs_utils::Componentable};

use super::{DrawLine, DrawLineList};

pub struct DrawLineCollection<V>(pub Vec<DrawLine<V>>);
impl<V> DrawLineCollection<V> {
	pub fn from_lines(lines: Vec<Line<V>>, color: Color) -> Self {
		Self(lines.into_iter().map(|line| DrawLine{line, color}).collect())
	}
}


pub struct DrawLineCollectionSystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for DrawLineCollectionSystem<V> 
where
	V: Componentable + Clone,
{
	type SystemData = (
		ReadStorage<'a, DrawLineCollection<V>>,
		WriteExpect<'a, DrawLineList<V>> // TODO: break up into components so that these can be processed more in parallel with par_iter?
	);

	fn run(&mut self, (
		line_collection_storage,
		mut lines
	) : Self::SystemData) {
		for lines_coll in line_collection_storage.join() {
			lines.0.extend(lines_coll.0.iter().map(|line| Some(line.clone())))
		}
	}
}