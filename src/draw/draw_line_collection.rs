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

	pub(crate) fn extend<I>(mut self, iter: I) -> Self
	where I: Iterator<Item = DrawLine<V>>
	{
		self.0.extend(iter);
		self
	}
}

pub fn draw_collection<'a, V: VectorTrait + 'a, I>(
	lines_collection: &DrawLineCollection<V>,
	shape_clip_state_iter: Option<I>, 
) -> Vec<Option<DrawLine<V>>>
where I: std::iter::Iterator<Item=&'a ShapeClipState<V>>
{  // TODO: return iterator?
	let lines = lines_collection.0.iter().map(|l| Some(l.clone())).collect(); // TODO: do we really need to clone here?
	match shape_clip_state_iter {
		Some(iter) => clip_draw_lines(
			lines,
			iter
		),
		None => lines,
	}
	
}