pub mod graphics2d;
pub mod graphics3d;

pub use graphics2d::Graphics2d;
pub use graphics3d::Graphics3d;
use crate::colors::Color;
//use glium::glutin;
use glium::Surface;
//use glium::Display;

//use glium::Surface;
//use glium::glutin::dpi::LogicalSize;
use glium::vertex::Vertex;

use crate::vector::{VectorTrait, MatrixTrait};
use crate::geometry::{VertIndex,Line};
use crate::draw::{DrawVertex,DrawLine};


pub trait Graphics {
	type VertexType : Vertex;
	type V : VectorTrait;

	const VERTEX_SHADER_SRC : &'static str;
	const FRAGMENT_SHADER_SRC : &'static str;
    const LINE_WIDTH : f32 = 2.0;

	fn new(display : glium::Display) -> Self ;

	fn get_display(&self) -> &glium::Display;
	fn get_vertex_buffer(&self) -> &glium::VertexBuffer<Self::VertexType>;
    fn get_index_buffer(&self) -> &glium::IndexBuffer<u16>;
    fn get_program(&self) -> &glium::Program;

    fn set_display(&mut self, display : glium::Display);
    fn set_vertex_buffer(&mut self, vertex_buffer : glium::VertexBuffer<Self::VertexType>);
    fn set_index_buffer(&mut self, index_buffer : glium::IndexBuffer<u16>);
    fn set_program(&mut self, program : glium::Program);

    fn new_vertex_buffer(&mut self, verts : &Vec<Option<DrawVertex<Self::V>>>) {
        self.set_vertex_buffer(
            glium::VertexBuffer::dynamic(self.get_display(),
            &Self::verts_to_gl(&verts))
            .unwrap()
            );
    }

    fn new_vertex_buffer_from_lines(&mut self, lines : &Vec<Option<DrawLine<Self::V>>>) {
        let vertexes = Self::opt_lines_to_gl(&lines);
        self.set_vertex_buffer(
            glium::VertexBuffer::dynamic(self.get_display(), &vertexes)
            .unwrap()
            );
    }
    fn new_index_buffer(&mut self, verts : &Vec<VertIndex>);

    fn vert_to_gl(vert : &Option<DrawVertex<Self::V>>) -> Self::VertexType;
	fn verts_to_gl(verts : &Vec<Option<DrawVertex<Self::V>>>) -> Vec<Self::VertexType> {
        verts.iter().map(Self::vert_to_gl)
            .collect()
    }
	fn vertis_to_gl(vertis : &Vec<VertIndex>) -> Vec<u16> {
    	vertis.iter().map(|v| *v as u16).collect()
	}
    fn opt_lines_to_gl(opt_lines : &Vec<Option<DrawLine<Self::V>>>) -> Vec<Self::VertexType>;
    
    fn build_perspective_mat<S : Surface>(target : &S) -> [[f32 ; 4] ; 4];

    fn build_view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
             f[2] * s_norm[0] - f[0] * s_norm[2],
             f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}
    fn draw_lines(&self, draw_lines : &Vec<Option<DrawLine<Self::V>>>) {

        
        //use VertexBufferAny? (use .into() )
        self.get_vertex_buffer().write(&Self::opt_lines_to_gl(&draw_lines));

        let draw_params = glium::DrawParameters{
            line_width : Some(2.0),
            smooth : Some(glium::draw_parameters::Smooth::Nicest),
            blend : glium::Blend::alpha_blending(),
            .. Default::default()
        };
        let mut target = self.get_display().draw();

        let view_matrix = match Self::V::DIM {
            2 => [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ 0.0, 0.0, 0.0 , 1.0f32],
            ],
            3 => Self::build_view_matrix(
                &[0.0,0.0,-4.0],
                &[0.0,0.0,1.0],
                &[0.0,1.0,0.0]
                ),
            _ => panic!("Invalid dimension")
        };
        let uniforms = uniform! {
            perspective : Self::build_perspective_mat(&target),
            view : view_matrix,
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ 0.0, 0.0, 0.0 , 1.0f32],
            ]
        };
        target.clear_color(0.0,0.0,0.0,1.0);
        target.draw(self.get_vertex_buffer(),
            &glium::index::NoIndices(glium::index::PrimitiveType::LinesList),
            self.get_program(),
            &uniforms,
            &draw_params).unwrap();

        target.finish().unwrap();

    }
}

