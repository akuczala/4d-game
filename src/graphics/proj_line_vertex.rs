use std::array::IntoIter;

use itertools::Itertools;

use super::{DrawLine, DrawVertex, VectorTrait, VertexTrait};

#[derive(Copy, Clone)]
pub struct NewVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub direction: f32,
    pub next: [f32; 3],
    pub previous: [f32; 3],
}
implement_vertex!(NewVertex, position, color, direction, next, previous);

impl Default for NewVertex {
    fn default() -> Self {
        Self::NO_DRAW
    }
}
impl NewVertex {
    fn project_pos<V: VectorTrait>(vertex: V) -> [f32; 3] {
        match V::DIM {
            2 => [vertex[0], vertex[1], 0.0],
            3 => [vertex[0], vertex[1], vertex[2]],
            _ => panic!("Invalid dimension"),
        } //as [f32 ; 3]
    }
}
impl VertexTrait for NewVertex {
    //make this consume its input?
    const NO_DRAW: Self = Self {
        position: [0., 0., 0.],
        color: [0., 0., 0., 0.],
        direction: 0.,
        next: [0., 0., 0.],
        previous: [0., 0., 0.],
    };
    //don't intend to use this function for this shader
    fn vert_to_gl<V: VectorTrait>(vert: &DrawVertex<V>) -> Self {
        let pos = NewVertex::project_pos(vert.vertex);
        Self {
            direction: 1.0,
            position: pos,
            color: *vert.color.get_arr(),
            next: pos,
            previous: pos,
        }
    }
    // TODO: a lot of time is spent in this fn
    fn line_to_gl<V: VectorTrait>(draw_line: &DrawLine<V>) -> Vec<Self> {
        Self::line_to_gl_iter(draw_line).collect_vec()
    }
    //type Iter = <Vec<Self> as IntoIterator>::IntoIter;
    type Iter = IntoIter<Self, 6>;
    fn line_to_gl_iter<V: VectorTrait>(draw_line: &DrawLine<V>) -> Self::Iter {
        Self::line_to_gl_arr(draw_line).into_iter()
    }

    fn line_to_gl_arr<V: VectorTrait>(draw_line: &DrawLine<V>) -> [Self; 6] {
        //Self::line_to_gl(maybe_line).into_iter()
        //let mut out = Vec::with_capacity(4);
        let draw_verts: [DrawVertex<V>; 2] = draw_line.get_draw_verts();
        //let dir = draw_verts[1].vertex - draw_verts[0].vertex;
        //let midpoint = crate::vector::VectorTrait::linterp(draw_verts[0].vertex,draw_verts[1].vertex,0.5);
        let proj_verts = draw_verts.map(|dv| NewVertex::project_pos(dv.vertex));
        //draw two triangles to make a line
        // if we really need to, we can have this return a fixed len array
        [(0, -1), (0, 1), (1, -1), (1, -1), (1, 1), (0, 1)].map(|(i, d)| Self {
            position: proj_verts[i],
            direction: d as f32,
            color: *draw_verts[i].color.get_arr(),
            next: proj_verts[1],
            previous: proj_verts[0],
        })
    }
}
