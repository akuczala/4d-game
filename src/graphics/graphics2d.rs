
use crate::graphics::VertexTrait;
use super::simple_vertex::SimpleVertex;
use crate::vector::{VectorTrait};
use crate::geometry::{VertIndex};
use super::Graphics;
use super::{VERTEX_SHADER_SRC,FRAGMENT_SHADER_SRC};
use crate::vector::{Vec2};
use glium::{Surface,Display};
use crate::draw::{DrawVertex,DrawLine};

#[derive(Copy, Clone)]
pub struct NewVertex {
    pub position: [f32; 3],
    pub color : [f32 ; 4],
    pub direction : f32,
    pub next : [f32 ; 3],
    pub previous : [f32 ; 3],
}
impl Default for NewVertex {
    fn default() -> Self {
        Self{
            position : [0.,0.,0.],
            color : [0.,0.,0.,0.],
            direction : 0.,
            next : [0.,0.,0.],
            previous : [0.,0.,0.],
        }
    }
}

implement_vertex!(NewVertex, position, color, direction, next, previous);

type Vertex = SimpleVertex;
pub struct Graphics2d {
    //display : &'a glium::Display,
    vertex_buffer : glium::VertexBuffer<Vertex>,
    index_buffer : glium::IndexBuffer<u16>, //can we change this to VertIndex=usize?
    program : glium::Program
}

impl Graphics<Vec2> for Graphics2d {
    type VertexType = Vertex;
    //type V = Vec2;

    const VERTEX_SHADER_SRC  : &'static str = VERTEX_SHADER_SRC;
    const FRAGMENT_SHADER_SRC  : &'static str = FRAGMENT_SHADER_SRC;

    //vertices are invisible at z = 10.0,
    //so they don't get drawn.
    //was originally using these for debugging
    const NO_DRAW : Self::VertexType = Self::VertexType::NO_DRAW;

    fn new(display : & glium::Display) -> Self {
        
        let program = glium::Program::from_source(display,
            VERTEX_SHADER_SRC,
            FRAGMENT_SHADER_SRC, None)
            .unwrap();
        let vertices : Vec<Self::VertexType> = Vec::new();
        let indices : Vec<u16> = Vec::new();
        let vertex_buffer = glium::VertexBuffer::dynamic(display, &vertices).unwrap();
        let index_buffer = glium::IndexBuffer::dynamic(
                display,
                glium::index::PrimitiveType::LinesList
                ,&indices
            )
            .unwrap();

        Self{
            //display,
            vertex_buffer,
            index_buffer,
            program
        }
    }

    // fn get_display(&self) -> &'a glium::Display {
    //     self.display
    // }
    fn get_vertex_buffer(&self) -> &glium::VertexBuffer<Self::VertexType> {
        &self.vertex_buffer
    }
    fn get_vertex_buffer_mut(&mut self) -> &mut glium::VertexBuffer<Self::VertexType> {
        &mut self.vertex_buffer
    }
    fn get_index_buffer(&self) -> &glium::IndexBuffer<u16> {
        &self.index_buffer
    }
    fn get_program(&self) -> &glium::Program {
        &self.program
    }

    // fn set_display(&mut self, display : &'a glium::Display) {
    //     self.display = display;
    // }
    fn set_vertex_buffer(&mut self, vertex_buffer : glium::VertexBuffer<Self::VertexType>) {
        self.vertex_buffer = vertex_buffer;
    }
    fn set_index_buffer(&mut self, index_buffer : glium::IndexBuffer<u16>) {
        self.index_buffer = index_buffer;
    }
    fn set_program(&mut self, program : glium::Program) {
        self.program = program;
    }

    fn new_index_buffer(&mut self, vertis : &Vec<VertIndex>, display : &Display) {
        self.index_buffer = glium::IndexBuffer::dynamic(
                display,
                glium::index::PrimitiveType::LinesList
                ,&&Self::vertis_to_gl(&vertis)
            )
            .unwrap();
    }
    // fn vert_to_gl(vert: &Option<DrawVertex<Vec2>>) -> Self::VertexType {
    //     Self::VertexType::vert_to_gl(vert)
    // }
    fn build_perspective_mat<S>(target : &S) -> [[f32 ; 4] ; 4]
    where S : Surface
    {
        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;
        let fov: f32 = 3.141592 / 3.0;
        //let zfar = 1024.0;
        //let znear = 0.1;

        let f = 1.0 / (fov / 2.0).tan();

        [
            [f*aspect_ratio, 0., 0., 0.],
            [0., f, 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.032f32]
        ]
    }
    
}

