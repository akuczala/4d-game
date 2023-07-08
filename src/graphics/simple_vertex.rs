use std::array::IntoIter;

use glium::index::PrimitiveType;
use itertools::Itertools;

use crate::vector::VecIndex;

use super::{DrawLine, DrawVertex, VectorTrait, VertexTrait};

pub const VERTEX_SHADER_SRC: &str = include_str!("simple-shader.vert");

// thicknesses greater than 1.0 seem to have no effect
const LINE_THICKNESS_3D: f32 = 1.0;
const LINE_THICKNESS_4D: f32 = 1.0;
// This, for some time now, cannot be used in place of NewVertex
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

fn line_to_gl_arr<V: VectorTrait>(line: &DrawLine<V>) -> [SimpleVertex; 2] {
    line.get_draw_verts().map(|v| SimpleVertex::vert_to_gl(&v))
}

impl VertexTrait for SimpleVertex {
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::LinesList;
    const VERTEX_SHADER_SRC: &'static str = VERTEX_SHADER_SRC;
    type Iter = IntoIter<Self, 2>;
    //make this consume its input?
    const NO_DRAW: Self = Self {
        position: [0.0, 0.0, 10.0],
        color: [1.0, 0.0, 0.0, 1.0f32],
    };
    fn line_thickness(dim: VecIndex) -> f32 {
        match dim {
            2 => LINE_THICKNESS_3D,
            3 => LINE_THICKNESS_4D,
            _ => panic!("Invalid dimension"),
        }
    }
    const LINE_BUFFER_SIZE: usize = 2;
    fn vert_to_gl<V: VectorTrait>(vert: &DrawVertex<V>) -> Self {
        let vertex = vert.vertex;
        Self {
            position: match V::DIM {
                2 => [vertex[0], vertex[1], 0.0],
                3 => [vertex[0], vertex[1], vertex[2]],
                _ => panic!("Invalid dimension"),
            } as [f32; 3],
            color: *vert.color.get_arr(),
        }
    }
    fn line_to_gl<V: VectorTrait>(draw_line: &DrawLine<V>) -> Vec<Self> {
        Self::line_to_gl_iter(draw_line).collect_vec()
    }
    fn line_to_gl_iter<V: VectorTrait>(draw_line: &DrawLine<V>) -> Self::Iter {
        //Self::line_to_gl(maybe_line).into_iter()
        line_to_gl_arr(draw_line).into_iter()
    }
}
implement_vertex!(SimpleVertex, position, color);
