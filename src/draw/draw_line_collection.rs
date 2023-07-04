use std::marker::PhantomData;

use itertools::Itertools;
use specs::{Join, ReadExpect, ReadStorage, System, WriteExpect};

use crate::{
    components::{ClipState, ShapeClipState},
    ecs_utils::Componentable,
    geometry::Line,
    graphics::colors::Color,
    vector::VectorTrait,
};

use super::{clipping::clip_draw_lines, DrawLine, DrawLineList};

pub struct DrawLineCollection<V>(pub Vec<DrawLine<V>>);
impl<V> DrawLineCollection<V> {
    pub fn from_lines(lines: Vec<Line<V>>, color: Color) -> Self {
        Self(
            lines
                .into_iter()
                .map(|line| DrawLine { line, color })
                .collect(),
        )
    }

    pub(crate) fn extend<I>(mut self, iter: I) -> Self
    where
        I: Iterator<Item = DrawLine<V>>,
    {
        self.0.extend(iter);
        self
    }
}

pub fn draw_collection<'a, V: VectorTrait + 'a, I>(
    lines_collection: &DrawLineCollection<V>,
    shape_clip_state_iter: Option<I>,
) -> Vec<Option<DrawLine<V>>>
where
    I: std::iter::Iterator<Item = &'a ShapeClipState<V>>,
{
    // TODO: return iterator?
    let lines = lines_collection.0.iter().map(|l| Some(l.clone())).collect(); // TODO: do we really need to clone here?
    match shape_clip_state_iter {
        Some(iter) => clip_draw_lines(lines, iter),
        None => lines,
    }
}
