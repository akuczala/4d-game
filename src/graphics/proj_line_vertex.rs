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
    fn vert_to_gl<V: VectorTrait>(vert: &Option<DrawVertex<V>>) -> Self {
        match *vert {
            Some(DrawVertex { vertex, color }) => {
                let pos = NewVertex::project_pos(vertex);
                Self {
                    direction: 1.0,
                    position: pos,
                    color: *color.get_arr(),
                    next: pos,
                    previous: pos,
                }
            }
            None => Self::NO_DRAW,
        }
    }
    // TODO: a lot of time is spent in this fn
    fn line_to_gl<V: VectorTrait>(maybe_line: &Option<DrawLine<V>>) -> Vec<Self> {
        match maybe_line {
            Some(draw_line) => {
                //let mut out = Vec::with_capacity(4);
                let draw_verts: [DrawVertex<V>; 2] = draw_line.get_draw_verts();
                //let dir = draw_verts[1].vertex - draw_verts[0].vertex;
                //let midpoint = crate::vector::VectorTrait::linterp(draw_verts[0].vertex,draw_verts[1].vertex,0.5);
                let proj_verts: Vec<[f32; 3]> = draw_verts
                    .iter()
                    .map(|dv| NewVertex::project_pos(dv.vertex))
                    .collect();
                //let proj_midpoint = NewVertex::project_pos(midpoint);
                // for &dir in [-1.,1. ].iter() {
                //     out.push(Self{
                //         position : proj_verts[0],
                //         direction : dir as f32,
                //         color : *draw_verts[0].color.get_arr(),
                //         next : proj_verts[1],
                //         previous : proj_verts[0],
                //     });
                //     out.push(Self{
                //         position : proj_verts[1],
                //         direction : dir as f32,
                //         color : *draw_verts[1].color.get_arr(),
                //         next : proj_verts[1],
                //         previous : proj_verts[0],
                //     })
                // }
                let mut out = vec![];
                //draw two triangles to make a line
                for &(i, d) in [(0, -1), (0, 1), (1, -1), (1, -1), (1, 1), (0, 1)].iter() {
                    out.push(Self {
                        position: proj_verts[i],
                        direction: d as f32,
                        color: *draw_verts[i].color.get_arr(),
                        next: proj_verts[1],
                        previous: proj_verts[0],
                    });
                }
                out
            }
            None => (0..6).map(|_| Self::NO_DRAW).collect(),
        }
    }
}
