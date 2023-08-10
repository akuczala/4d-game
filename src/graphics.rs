use glium::buffer::WriteMapping;
use glium::vertex::Vertex;
use glium::Display;

use glium::Surface;
use glium::VertexBuffer;

use crate::constants::BACKGROUND_COLOR;
use crate::draw::{DrawLine, DrawVertex};

use crate::vector::VecIndex;
use crate::vector::VectorTrait;

use self::matrices::build_perspective_matrix;
use self::matrices::build_view_matrix;
use self::matrices::IDENTITY_MATRIX;
use self::proj_line_vertex::ProjLineVertex;

pub mod colors;
mod matrices;
mod proj_line_vertex;
mod simple_vertex;

const FRAGMENT_SHADER_SRC: &str = include_str!("graphics/simple-shader.frag");

pub trait VertexTrait: Vertex {
    const NO_DRAW: Self;
    const LINE_BUFFER_SIZE: usize;
    const PRIMITIVE_TYPE: glium::index::PrimitiveType;
    const VERTEX_SHADER_SRC: &'static str;
    type Iter: Iterator<Item = Self>;
    fn vert_to_gl<V: VectorTrait>(vert: &DrawVertex<V>) -> Self;
    fn line_to_gl<V: VectorTrait>(draw_line: &DrawLine<V>) -> Vec<Self>;
    fn line_to_gl_iter<V: VectorTrait>(draw_line: &DrawLine<V>) -> Self::Iter;
    fn line_thickness(dim: VecIndex) -> f32;
    // this may be slightly(?) faster than line_to_gl_iter, but i don't know how to make the arr size generic
    //fn line_to_gl_arr<V: VectorTrait>(maybe_line: &DrawLine<V>) -> [Self; 6];
}

pub trait GraphicsTrait {
    fn init(display: &Display) -> Self;
    fn draw_lines<V: VectorTrait>(
        &mut self,
        draw_lines: &[DrawLine<V>],
        target: glium::Frame,
    ) -> glium::Frame;
    fn update_buffer<V: VectorTrait>(&mut self, draw_lines: &[DrawLine<V>], display: &Display);
}

pub type DefaultGraphics = Graphics<ProjLineVertex>;
pub struct Graphics<X: Copy> {
    pub vertex_buffer: glium::VertexBuffer<X>,
    pub index_buffer: glium::IndexBuffer<u16>, //can we change this to VertIndex=usize?
    pub program: glium::Program,
    cur_lines_len: usize, // we store buffer size here because apparently calling vertex_buffer.len() is expensive
}
impl<X: VertexTrait> Graphics<X> {
    pub fn new(display: &glium::Display) -> Self {
        Self {
            vertex_buffer: glium::VertexBuffer::dynamic(display, &Vec::new()).unwrap(),
            index_buffer: glium::IndexBuffer::dynamic(
                display,
                glium::index::PrimitiveType::LinesList,
                &Vec::new(),
            )
            .unwrap(),
            program: glium::Program::from_source(
                display,
                X::VERTEX_SHADER_SRC,
                FRAGMENT_SHADER_SRC,
                None,
            )
            .unwrap(),
            cur_lines_len: 0,
        }
    }
}

fn opt_lines_to_gl<X: VertexTrait, V: VectorTrait>(opt_lines: &[DrawLine<V>]) -> Vec<X> {
    opt_lines.iter().flat_map(X::line_to_gl).collect()
}

fn new_vertex_buffer_from_lines<X: VertexTrait, V: VectorTrait>(
    lines: &[DrawLine<V>],
    display: &Display,
) -> VertexBuffer<X> {
    let vertexes = opt_lines_to_gl(lines);
    glium::VertexBuffer::dynamic(display, &vertexes).unwrap()
}

fn write_opt_lines_to_buffer<X: VertexTrait, V: VectorTrait>(
    write_map: &mut WriteMapping<[X]>,
    buffer_len: usize,
    opt_lines: &[DrawLine<V>],
) {
    // TODO: this could be refactored with flat_map etc but i don't know how that impacts performance
    // TODO: is it faster to call .write once rather than write_map.set a bunch of times?
    let mut i = 0;
    for opt_line in opt_lines.iter() {
        for v in X::line_to_gl_iter(opt_line) {
            write_map.set(i, v);
            i += 1;
        }
    }
    // Set remaining buffer with NO_DRAW verts to avoid "ghosts" while keeping buffer len unchanged
    for j in i..buffer_len {
        write_map.set(j, X::NO_DRAW);
    }
}

impl<X: VertexTrait> GraphicsTrait for Graphics<X> {
    fn init(display: &Display) -> Self {
        Self::new(display)
    }

    fn update_buffer<V: VectorTrait>(&mut self, draw_lines: &[DrawLine<V>], display: &Display) {
        //make new buffer if
        // a. the number of lines increases (need more room in the buffer)
        // b. the number of lines drastically decreases (to not waste memory)
        let cur_lines_len = self.cur_lines_len;
        let draw_lines_len = draw_lines.len();
        if (draw_lines_len > cur_lines_len) | (draw_lines_len < cur_lines_len / 2) {
            self.vertex_buffer = new_vertex_buffer_from_lines(draw_lines, display);
            // println!(
            //     "New buffer! {} to {}",
            //     self.cur_lines_len, draw_lines_len
            // );
            self.cur_lines_len = draw_lines_len;
        }
    }

    fn draw_lines<V: VectorTrait>(
        &mut self,
        draw_lines: &[DrawLine<V>],
        mut target: glium::Frame,
    ) -> glium::Frame {
        //self.get_vertex_buffer().write(&Self::opt_lines_to_gl(&draw_lines));
        write_opt_lines_to_buffer(
            &mut self.vertex_buffer.map_write(),
            self.cur_lines_len * X::LINE_BUFFER_SIZE,
            draw_lines,
        ); //slightly faster than the above (less allocation)

        let draw_params = glium::DrawParameters {
            smooth: Some(glium::draw_parameters::Smooth::Nicest),
            blend: glium::Blend::alpha_blending(), //lines are a lot darker
            line_width: Some(X::line_thickness(V::DIM)),
            ..Default::default()
        };
        let (width, height) = target.get_dimensions();
        let uniforms = uniform! {
            perspective : build_perspective_matrix(V::DIM, width, height),
            view : build_view_matrix(V::DIM),
            model: IDENTITY_MATRIX,
            // below needed only for proj_line vertex
            aspect : (width as f32)/(height as f32),
            thickness : X::line_thickness(V::DIM),
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
                &self.vertex_buffer,
                glium::index::NoIndices(X::PRIMITIVE_TYPE),
                &self.program,
                &uniforms,
                &draw_params,
            )
            .unwrap();

        target
    }
}
