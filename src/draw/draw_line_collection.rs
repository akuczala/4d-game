use crate::{
    components::ShapeClipState, geometry::Line, graphics::colors::Color, vector::VectorTrait,
};

use super::{clipping::clip_draw_lines, DrawLine, Scratch};

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
    write_lines: &mut Vec<DrawLine<V>>,
    line_scratch: &mut Scratch<Line<V>>,
    lines_collection: &DrawLineCollection<V>,
    shape_clip_state_iter: Option<I>,
) where
    I: std::iter::Iterator<Item = &'a ShapeClipState<V>>,
{
    match shape_clip_state_iter {
        Some(iter) => clip_draw_lines(&lines_collection.0, write_lines, line_scratch, iter),
        None => write_lines.extend(lines_collection.0.iter().cloned()),
    }
}
