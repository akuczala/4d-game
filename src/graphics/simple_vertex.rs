use super::{DrawLine, DrawVertex, VectorTrait, VertexTrait};

#[derive(Copy, Clone)]
pub struct SimpleVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}
impl Default for SimpleVertex {
    fn default() -> Self {
        Self::NO_DRAW
    }
}
impl VertexTrait for SimpleVertex {
    //make this consume its input?
    const NO_DRAW: Self = Self {
        position: [0.0, 0.0, 10.0],
        color: [1.0, 0.0, 0.0, 1.0f32],
    };
    fn vert_to_gl<V: VectorTrait>(vert: &Option<DrawVertex<V>>) -> Self {
        match *vert {
            Some(DrawVertex { vertex, color }) => Self {
                position: match V::DIM {
                    2 => [vertex[0], vertex[1], 0.0],
                    3 => [vertex[0], vertex[1], vertex[2]],
                    _ => panic!("Invalid dimension"),
                } as [f32; 3],
                color: *color.get_arr(),
            },
            None => Self::NO_DRAW,
        }
    }
    fn line_to_gl<V: VectorTrait>(maybe_line: &Option<DrawLine<V>>) -> Vec<Self> {
        match maybe_line {
            Some(draw_line) => draw_line
                .get_draw_verts()
                .iter()
                .map(|&v| Self::vert_to_gl(&Some(v)))
                .collect(),
            None => vec![Self::NO_DRAW, Self::NO_DRAW],
        }
    }
}
implement_vertex!(SimpleVertex, position, color);
