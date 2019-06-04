pub mod graphics2d;
//pub mod graphics3d;

pub use graphics2d::Graphics2d;
use crate::colors::Color;
//use glium::glutin;
use glium::Surface;
//use glium::Display;

//use glium::Surface;
//use glium::glutin::dpi::LogicalSize;
use glium::vertex::Vertex;

use crate::vector::{VectorTrait};
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
    
    fn build_perspective_mat<S>(target : &S) -> [[f32 ; 4] ; 4]
    where S : Surface;
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

        let uniforms = uniform! {
            perspective : Self::build_perspective_mat(&target),
            matrix: [
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

