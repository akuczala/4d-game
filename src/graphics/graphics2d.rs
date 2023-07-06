use std::f32::consts::PI;

use super::proj_line_vertex::NewVertex;
use super::simple_vertex::SimpleVertex;
use super::{FRAGMENT_SHADER_SRC, VERTEX_SHADER_SRC};
use crate::geometry::shape::VertIndex;
use crate::graphics::VertexTrait;
use crate::vector::Vec2;
use glium::{Display, Surface};

pub fn build_perspective_mat_2d<S>(target: &S) -> [[f32; 4]; 4]
where
    S: Surface,
{
    let (width, height) = target.get_dimensions();
    let aspect_ratio = height as f32 / width as f32;
    let fov: f32 = PI / 3.0;
    //let zfar = 1024.0;
    //let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [f * aspect_ratio, 0., 0., 0.],
        [0., f, 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.032f32],
    ]
}
