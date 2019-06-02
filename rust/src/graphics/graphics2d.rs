
use crate::vector::{VectorTrait,MatrixTrait};
use crate::geometry::{VertIndex,Line};
use super::Graphics;
use crate::vector::{Vec2,Vec3};



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
    uniform mat4 matrix;

    void main() {
        in_color = color;
        gl_Position = perspective * matrix * vec4(position, 1.0);
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
pub struct Graphics2d {
    display : glium::Display,
    vertex_buffer : glium::VertexBuffer<Vertex>,
    index_buffer : glium::IndexBuffer<u16>, //can we change this to VertIndex=usize?
    program : glium::Program
}

const BEHIND : Vertex = Vertex{
    position : [0.0,0.0,0.5],
    color : [1.0,0.0,0.0,0.0f32]
};
const BEHIND2 : Vertex = Vertex{
    position : [0.0,10.0,0.5],
    color : [1.0,0.0,0.0,1.0f32]
};

impl Graphics for Graphics2d {
    type VertexType = Vertex;
    type V = Vec2;

    const VERTEX_SHADER_SRC  : &'static str = VERTEX_SHADER_SRC;
    const FRAGMENT_SHADER_SRC  : &'static str = FRAGMENT_SHADER_SRC;

    fn new(display : glium::Display) -> Self {
        
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
            display,
            vertex_buffer,
            index_buffer,
            program
        }
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
    fn new_vertex_buffer_from_lines(&mut self, lines : &Vec<Option<Line<Self::V>>>) {
        let vertexes = Self::opt_line_to_gl(&lines);
        self.vertex_buffer = glium::VertexBuffer::dynamic(&self.display, &vertexes).unwrap();
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
        verts.iter().map(Self::vert_to_gl)
            .collect()
    }
    fn vert_to_gl(vert: &Vec2) -> Vertex {
        let arr = *vert.get_arr() ;
        Vertex{
            position : [arr[0],arr[1],0.0] as [f32 ; 3],
            color : [1.0,1.0,1.0,1.0f32]
        }
    }
    fn vert_to_gl_red(vert: &Vec2) -> Vertex {
        let arr = *vert.get_arr() ;
        Vertex{
            position : [arr[0],arr[1],0.0] as [f32 ; 3],
            color : [1.0,0.0,0.0,0.0f32]
        }
    }
    fn opt_line_to_gl(opt_lines: &Vec<Option<Line<Vec2>>>) -> Vec<Vertex> {
        let mut verts : Vec<Vertex> = Vec::new();
        for opt_line in opt_lines.iter() {
            let (v0,v1) = match opt_line {
                Some(Line(v0,v1)) => (Self::vert_to_gl(v0),Self::vert_to_gl(v1)),
                None => (BEHIND,BEHIND2)
            };
            verts.push(v0); verts.push(v1);
        }
        verts
    }
}

