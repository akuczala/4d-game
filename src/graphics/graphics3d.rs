use std::f32::consts::PI;

use super::{proj_line_vertex::NewVertex, FRAGMENT_SHADER_SRC, VERTEX_SHADER_SRC};
use crate::geometry::shape::VertIndex;
use glium::{Display, Surface};

pub fn build_perspective_mat_3d<S>(target: &S) -> [[f32; 4]; 4]
where
    S: Surface,
{
    let (width, height) = target.get_dimensions();
    let aspect_ratio = height as f32 / width as f32;
    //let fov: f32 = 3.141592 / 3.0; //nearly fish eye
    //let fov : f32 = 3.141592 / 8.0;
    let fov: f32 = PI / 16.0; //comparable to 3d
                              //let zfar = 1024.0;/
    let zfar = 100.0;
    let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();
    [
        [f * aspect_ratio, 0., 0., 0.],
        [0., f, 0., 0.],
        [0., 0., (zfar + znear) / (zfar - znear), 1.0],
        [0., 0., 0., 1.032f32],
    ]
}
