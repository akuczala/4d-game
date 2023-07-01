use std::marker::PhantomData;

use itertools::Itertools;
use specs::{ReadStorage, WriteExpect, System, Join, ReadExpect};

use crate::{graphics::colors::Color, geometry::Line, vector::VectorTrait, ecs_utils::Componentable, components::{ClipState, ShapeClipState}};

use super::{DrawLine, DrawLineList, clipping::clip_draw_lines};

pub struct DrawLineCollection<V>(pub Vec<DrawLine<V>>);
impl<V> DrawLineCollection<V> {
	pub fn from_lines(lines: Vec<Line<V>>, color: Color) -> Self {
		Self(lines.into_iter().map(|line| DrawLine{line, color}).collect())
	}
}


pub struct DrawLineCollectionSystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for DrawLineCollectionSystem<V> 
where
	V: VectorTrait + Componentable,
{
	type SystemData = (
		ReadStorage<'a, DrawLineCollection<V>>,
		ReadStorage<'a, ShapeClipState<V>>,
		ReadExpect<'a, ClipState<V>>,
		WriteExpect<'a, DrawLineList<V>> // TODO: break up into components so that these can be processed more in parallel with par_iter?
	);

	fn run(&mut self, (
		line_collection_storage,
		read_shape_clip_state,
		clip_state,
		mut lines
	) : Self::SystemData) {
		for lines_coll in line_collection_storage.join() {
			lines.0.extend(draw_collection(lines_coll, &read_shape_clip_state, clip_state.clipping_enabled));
		}
	}
}

pub fn draw_collection<V: VectorTrait + Componentable>(
	lines_collection: &DrawLineCollection<V>,
	read_shape_clip_state: &ReadStorage<ShapeClipState<V>>, // TODO: take iterator instead of readstorage?
	clipping_enabled: bool
) -> Vec<Option<DrawLine<V>>> {
	let lines = lines_collection.0.iter().map(|l| Some(l.clone())).collect(); // TODO: do we really need to clone here?
	if clipping_enabled {
		clip_draw_lines(
			lines,
			(&read_shape_clip_state).join()
		) // TODO: return iterator?
	} else {
		lines
	}
	
}