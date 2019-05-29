
use super::listen_events;

use crate::vector::{VectorTrait,MatrixTrait};
use crate::geometry::VertIndex;
use super::Graphics;


#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}
implement_vertex!(Vertex, position);

pub const VERTEX_SHADER_SRC : &str = r#"
    #version 140

    in vec2 position;
    out vec2 my_attr;

    uniform mat4 perspective;
    uniform mat4 matrix;

    void main() {
        my_attr = position;
        gl_Position = perspective * matrix * vec4(position, 0.0, 1.0);
    }
    "#;
pub const FRAGMENT_SHADER_SRC : &str = r#"
    #version 140

    in vec2 my_attr;
    out vec4 color;

    void main() {
        color = vec4(1.0,1.0,1.0,1.0);
    }
    "#;
pub struct Graphics2d {
    events_loop : winit::EventsLoop,
    display : glium::Display,
    vertex_buffer : glium::VertexBuffer<Vertex>,
    index_buffer : glium::IndexBuffer<u16>, //can we change this to VertIndex=usize?
    program : glium::Program
}
impl Graphics for Graphics2d {
    type VertexType = Vertex;
    type V = Vec2;

    const VERTEX_SHADER_SRC  : &'static str = VERTEX_SHADER_SRC;
    const FRAGMENT_SHADER_SRC  : &'static str = FRAGMENT_SHADER_SRC;

    fn new() -> Self {
        let (events_loop,display) = Self::init_glium();
        let program = glium::Program::from_source(&display,
            VERTEX_SHADER_SRC,
            FRAGMENT_SHADER_SRC, None)
            .unwrap();
        let vertices : Vec<Vertex> = Vec::new();
        let indices : Vec<u16> = Vec::new();
        let vertex_buffer = glium::VertexBuffer::dynamic(&display, &vertices).unwrap();
        let index_buffer = glium::IndexBuffer::dynamic(
                &display,
                glium::index::PrimitiveType::LinesList
                ,&indices
            )
            .unwrap();

        Self{
            events_loop : events_loop,
            display : display,
            vertex_buffer : vertex_buffer,
            index_buffer : index_buffer,
            program : program
        }
    }

    fn get_event_loop(&mut self) -> &mut winit::EventsLoop {
        &mut self.events_loop
    }
    fn get_display(&self) -> &glium::Display {
        &self.display
    }
    fn get_vertex_buffer(&self) -> &glium::VertexBuffer<Self::VertexType> {
        &self.vertex_buffer
    }
    fn get_index_buffer(&self) -> &glium::IndexBuffer<u16> {
        &self.index_buffer
    }
    fn get_program(&self) -> &glium::Program {
        &self.program
    }

    fn new_vertex_buffer(&mut self, verts : &Vec<Self::V>) {
        self.vertex_buffer = glium::VertexBuffer::dynamic(&self.display, &Self::verts_to_gl(&verts)).unwrap();
    }
    fn new_index_buffer(&mut self, vertis : &Vec<VertIndex>) {
        self.index_buffer = glium::IndexBuffer::dynamic(
                &self.display,
                glium::index::PrimitiveType::LinesList
                ,&&Self::vertis_to_gl(&vertis)
            )
            .unwrap();
    }

    fn verts_to_gl(verts : &Vec<Vec2>) -> Vec<Vertex> {
        verts.iter().map(|v| Vertex{position : *v.get_arr() as [f32 ; 2]})
            .collect()
    }
}

use crate::vector::{Vec2};

