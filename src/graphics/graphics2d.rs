
use crate::vector::{VectorTrait};
use crate::geometry::{VertIndex};
use super::Graphics;
use crate::vector::{Vec2};
use glium::Surface;
use crate::draw::{DrawVertex,DrawLine};


#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color : [f32 ; 4]
}
implement_vertex!(Vertex, position, color);

pub const VERTEX_SHADER_SRC : &str = r#"
    #version 140

    in vec3 position;
    in vec4 color;
    out vec4 in_color;
    uniform mat4 perspective;
    uniform mat4 view;
    uniform mat4 model;

    void main() {
        in_color = color;
        gl_Position = perspective * view * model * vec4(position, 1.0);
    }
    "#;
pub const FRAGMENT_SHADER_SRC : &str = r#"
    #version 140

    in vec4 in_color;
    out vec4 color;

    void main() {
        color = vec4(in_color);
    }
    "#;
pub struct Graphics2d<'a> {
    display : &'a glium::Display,
    vertex_buffer : glium::VertexBuffer<Vertex>,
    index_buffer : glium::IndexBuffer<u16>, //can we change this to VertIndex=usize?
    program : glium::Program
}


impl<'a> Graphics<'a,Vec2> for Graphics2d<'a> {
    type VertexType = Vertex;
    //type V = Vec2;

    const VERTEX_SHADER_SRC  : &'static str = VERTEX_SHADER_SRC;
    const FRAGMENT_SHADER_SRC  : &'static str = FRAGMENT_SHADER_SRC;

    //vertices are invisible at z = 10.0,
    //so they don't get drawn.
    //was originally using these for debugging
    const NO_DRAW : Vertex = Vertex{
        position : [0.0,0.0,10.0],
        color : [1.0,0.0,0.0,1.0f32]
    };

    fn new(display : &'a glium::Display) -> Self {
        
        let program = glium::Program::from_source(display,
            VERTEX_SHADER_SRC,
            FRAGMENT_SHADER_SRC, None)
            .unwrap();
        let vertices : Vec<Vertex> = Vec::new();
        let indices : Vec<u16> = Vec::new();
        let vertex_buffer = glium::VertexBuffer::dynamic(display, &vertices).unwrap();
        let index_buffer = glium::IndexBuffer::dynamic(
                display,
                glium::index::PrimitiveType::LinesList
                ,&indices
            )
            .unwrap();

        Self{
            display,
            vertex_buffer,
            index_buffer,
            program
        }
    }

    fn get_display(&self) -> &'a glium::Display {
        self.display
    }
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

    fn set_display(&mut self, display : &'a glium::Display) {
        self.display = display;
    }
    fn set_vertex_buffer(&mut self, vertex_buffer : glium::VertexBuffer<Self::VertexType>) {
        self.vertex_buffer = vertex_buffer;
    }
    fn set_index_buffer(&mut self, index_buffer : glium::IndexBuffer<u16>) {
        self.index_buffer = index_buffer;
    }
    fn set_program(&mut self, program : glium::Program) {
        self.program = program;
    }

    fn new_index_buffer(&mut self, vertis : &Vec<VertIndex>) {
        self.index_buffer = glium::IndexBuffer::dynamic(
                self.display,
                glium::index::PrimitiveType::LinesList
                ,&&Self::vertis_to_gl(&vertis)
            )
            .unwrap();
    }

    //make this consume its input?
    fn vert_to_gl(vert: &Option<DrawVertex<Vec2>>) -> Vertex {
        match *vert {
            Some(DrawVertex{vertex,color}) => {
                let arr = *vertex.get_arr() ;
                Vertex{
                    position : [arr[0],arr[1],0.0] as [f32 ; 3],
                    color : *color.get_arr()
                }

            }
            None => Self::NO_DRAW
        }
        
    }

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
