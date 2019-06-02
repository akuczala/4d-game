pub mod graphics2d;
//pub mod graphics3d;

pub use graphics2d::Graphics2d;
use glium::glutin;
use glium::Surface;
use glium::Display;

//use glium::Surface;
use glium::glutin::dpi::LogicalSize;
use glium::vertex::Vertex;

use crate::vector::{VectorTrait};
use crate::geometry::{VertIndex,Line};



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

    fn new_vertex_buffer(&mut self, verts : &Vec<Self::V>);
    fn new_vertex_buffer_from_lines(&mut self, lines : &Vec<Option<Line<Self::V>>>);
    fn new_index_buffer(&mut self, verts : &Vec<VertIndex>);

    fn vert_to_gl(verts : &Self::V) -> Self::VertexType;
    fn vert_to_gl_red(verts : &Self::V) -> Self::VertexType;
	fn verts_to_gl(verts : &Vec<Self::V>) -> Vec<Self::VertexType>;
	fn vertis_to_gl(vertis : &Vec<VertIndex>) -> Vec<u16> {
    	vertis.iter().map(|v| *v as u16).collect()
	}
    fn opt_line_to_gl(opt_lines : &Vec<Option<Line<Self::V>>>) -> Vec<Self::VertexType>;
    fn draw_lines(&self, lines : &Vec<Option<Line<Self::V>>>) {

        let perspective = [
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.032f32]];
        let uniforms = uniform! {
            perspective : perspective,
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ 0.0, 0.0, 0.0 , 1.0f32],
            ]
        };
        //use VertexBufferAny? (use .into() )
        self.get_vertex_buffer().write(&Self::opt_line_to_gl(&lines));

        let draw_params = glium::DrawParameters{
            line_width : Some(2.0),
            smooth : Some(glium::draw_parameters::Smooth::Nicest),
            blend : glium::Blend::alpha_blending(),
            .. Default::default()
        };
        let mut target = self.get_display().draw();
        target.clear_color(0.0,0.1,0.1,1.0);
        target.draw(self.get_vertex_buffer(),
            &glium::index::NoIndices(glium::index::PrimitiveType::LinesList),
            self.get_program(),
            &uniforms,
            &draw_params).unwrap();

        target.finish().unwrap();

    }
	fn draw_lines_old(&self, verts : &Vec<Self::V>, vertis : &Vec<VertIndex>) {

		let perspective = [
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.032f32]];
        let uniforms = uniform! {
            perspective : perspective,
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ 0.0, 0.0, 0.0 , 1.0f32],
            ]
        };
        //use VertexBufferAny? (use .into() )
		self.get_vertex_buffer().write(&Self::verts_to_gl(&verts));
        self.get_index_buffer().write(&Self::vertis_to_gl(&vertis));

        let draw_params = glium::DrawParameters{
            line_width : Some(2.0),
            smooth : Some(glium::draw_parameters::Smooth::Nicest),
            blend : glium::Blend::alpha_blending(),
            .. Default::default()
        };
        let mut target = self.get_display().draw();
        target.clear_color(0.0,0.1,0.1,1.0);
        target.draw(self.get_vertex_buffer(),
            self.get_index_buffer(),
            self.get_program(),
            &uniforms,
            &draw_params).unwrap();

        target.finish().unwrap();

	}
}

