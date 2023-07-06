use glium::vertex::Vertex;
use glium::Display;
use glium::IndexBuffer;
use glium::Surface;
use glium::VertexBuffer;

use crate::constants::BACKGROUND_COLOR;
use crate::constants::LINE_THICKNESS_3D;
use crate::constants::LINE_THICKNESS_4D;
use crate::draw::{DrawLine, DrawVertex};
use crate::geometry::shape::VertIndex;
use crate::vector::VectorTrait;

use self::graphics2d::build_perspective_mat_2d;
use self::graphics3d::build_perspective_mat_3d;

pub mod colors;
pub mod graphics2d;
pub mod graphics3d;
pub mod proj_line_vertex;
mod simple_vertex;

//pub const VERTEX_SHADER_SRC : &str = include_str!("graphics/test-shader.vert");
pub const VERTEX_SHADER_SRC: &str = include_str!("graphics/test-shader.vert");
pub const FRAGMENT_SHADER_SRC: &str = include_str!("graphics/simple-shader.frag");

pub trait VertexTrait: Vertex {
    const NO_DRAW: Self;
    fn vert_to_gl<V: VectorTrait>(vert: &Option<DrawVertex<V>>) -> Self;
    fn line_to_gl<V: VectorTrait>(maybe_line: &Option<DrawLine<V>>) -> Vec<Self>;
}

pub struct Graphics<X: Copy> {
    pub vertex_buffer: glium::VertexBuffer<X>,
    pub index_buffer: glium::IndexBuffer<u16>, //can we change this to VertIndex=usize?
    pub program: glium::Program,
}
impl<X: Vertex> Graphics<X> {
    pub fn new(display: &glium::Display) -> Self {
        let program =
            glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap();
        let vertices: Vec<X> = Vec::new();
        let indices: Vec<u16> = Vec::new();
        let vertex_buffer = glium::VertexBuffer::dynamic(display, &vertices).unwrap();
        let index_buffer =
            glium::IndexBuffer::dynamic(display, glium::index::PrimitiveType::LinesList, &indices)
                .unwrap();

        Self {
            vertex_buffer,
            index_buffer,
            program,
        }
    }
}

fn verts_to_gl<X: VertexTrait, V: VectorTrait>(verts: &[Option<DrawVertex<V>>]) -> Vec<X> {
    verts.iter().map(X::vert_to_gl).collect()
}
fn vertis_to_gl(vertis: &[VertIndex]) -> Vec<u16> {
    vertis.iter().map(|v| *v as u16).collect()
}
fn opt_lines_to_gl<X: VertexTrait, V: VectorTrait>(opt_lines: &[Option<DrawLine<V>>]) -> Vec<X> {
    opt_lines.iter().flat_map(X::line_to_gl).collect()
}

fn new_index_buffer(vertis: &[VertIndex], display: &Display) -> IndexBuffer<u16> {
    glium::IndexBuffer::dynamic(
        display,
        glium::index::PrimitiveType::LinesList,
        &vertis_to_gl(vertis),
    )
    .unwrap()
}

fn new_vertex_buffer<X: VertexTrait, V: VectorTrait>(
    verts: &[Option<DrawVertex<V>>],
    display: &Display,
) -> VertexBuffer<X> {
    glium::VertexBuffer::dynamic(display, &verts_to_gl(verts)).unwrap()
}

pub fn new_vertex_buffer_from_lines<X: VertexTrait, V: VectorTrait>(
    lines: &[Option<DrawLine<V>>],
    display: &Display,
) -> VertexBuffer<X> {
    let vertexes = opt_lines_to_gl(lines);
    glium::VertexBuffer::dynamic(display, &vertexes).unwrap()
}

fn write_opt_lines_to_buffer<X: VertexTrait, V: VectorTrait>(
    vertex_buffer: &mut VertexBuffer<X>,
    opt_lines: &[Option<DrawLine<V>>],
) {
    let mut write_map = vertex_buffer.map_write();

    // TODO: this could be refactored with flat_map etc but i don't know how that impacts performance
    //we might avoid allocating a vec if we have line_to_gl return an iterator
    // this is a fn that eats a fair amount of performance (particularly due to vec allocation)
    let mut i = 0;
    for opt_line in opt_lines.iter() {
        for &v in X::line_to_gl(opt_line).iter() {
            write_map.set(i, v);
            i += 1;
        }
    }
    // Set remaining buffer with NO_DRAW verts to avoid "ghosts" while keeping buffer len unchanged
    for j in i..write_map.len() {
        write_map.set(j, X::NO_DRAW);
    }
}

fn build_view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [
        up[1] * f[2] - up[2] * f[1],
        up[2] * f[0] - up[0] * f[2],
        up[0] * f[1] - up[1] * f[0],
    ];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [
        f[1] * s_norm[2] - f[2] * s_norm[1],
        f[2] * s_norm[0] - f[0] * s_norm[2],
        f[0] * s_norm[1] - f[1] * s_norm[0],
    ];

    let p = [
        -position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
        -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
        -position[0] * f[0] - position[1] * f[1] - position[2] * f[2],
    ];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

pub fn update_buffer<X: VertexTrait, V: VectorTrait>(
    graphics: &mut Graphics<X>,
    draw_lines: &[Option<DrawLine<V>>],
    display: &Display,
) {
    //make new buffer if
    // a. the number of lines increases (need more room in the buffer)
    // b. the number of lines drastically decreases (to not waste memory)
    let cur_lines_len = graphics.vertex_buffer.len();
    let draw_lines_len = draw_lines.len();
    if (draw_lines_len > cur_lines_len) | (draw_lines_len < cur_lines_len / 2) {
        graphics.vertex_buffer = new_vertex_buffer_from_lines(draw_lines, display);
        // println!(
        //     "New buffer! {} to {}",
        //     self.cur_lines_length, draw_lines_len
        // );
    }
}

pub fn draw_lines<X: VertexTrait, V: VectorTrait>(
    graphics: &mut Graphics<X>,
    draw_lines: &[Option<DrawLine<V>>],
    mut target: glium::Frame,
) -> glium::Frame {
    //self.get_vertex_buffer().write(&Self::opt_lines_to_gl(&draw_lines));
    write_opt_lines_to_buffer(&mut graphics.vertex_buffer, draw_lines); //slightly faster than the above (less allocation)

    let draw_params = glium::DrawParameters {
        smooth: Some(glium::draw_parameters::Smooth::Nicest),
        blend: glium::Blend::alpha_blending(), //lines are a lot darker
        ..Default::default()
    };
    //let mut target = display.draw();
    let (width, height) = target.get_dimensions();
    let view_matrix = match V::DIM {
        2 => [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ],
        3 => build_view_matrix(&[2.0, 2.0, -4.0], &[-1.0, -1.0, 2.0], &[0.0, 1.0, 0.0]),
        _ => panic!("Invalid dimension"),
    };
    let uniforms = uniform! {
        perspective : match V::DIM {
            2 => build_perspective_mat_2d(&target),
            3 => build_perspective_mat_3d(&target),
            _ => panic!("Invalid dimension")
        },
        view : view_matrix,
        model: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [ 0.0, 0.0, 0.0 , 1.0f32],
        ],
        aspect : (width as f32)/(height as f32),
        thickness : match V::DIM {
            2 =>LINE_THICKNESS_3D, 3 => LINE_THICKNESS_4D,
            _ => panic!("Invalid dimension")},
        miter : 1,
    };
    target.clear_color(
        BACKGROUND_COLOR[0],
        BACKGROUND_COLOR[1],
        BACKGROUND_COLOR[2],
        BACKGROUND_COLOR[3],
    );
    target
        .draw(
            &graphics.vertex_buffer,
            glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
            &graphics.program,
            &uniforms,
            &draw_params,
        )
        .unwrap();

    target
}
