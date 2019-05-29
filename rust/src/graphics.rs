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
use crate::geometry::VertIndex;

pub struct ButtonsPressed {
	pub w : bool,
	pub s : bool,
	pub a : bool,
	pub d : bool,
	pub space : bool
}
impl ButtonsPressed {
	pub fn new() -> Self{
		ButtonsPressed {
			w: false, s: false, a : false, d : false, space : false
		}
	}
}

use glutin::VirtualKeyCode as VKC;
use glutin::ElementState::{Pressed,Released};
// listing the events produced by application and waiting to be received
pub fn listen_events(events_loop :  &mut winit::EventsLoop, closed: &mut bool, pressed : &mut ButtonsPressed) {
    events_loop.poll_events(|ev| {
            match ev {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => *closed = true,
                    glutin::WindowEvent::KeyboardInput{input, ..} => match input {
                    	glutin::KeyboardInput{ virtual_keycode, state, ..} => match (virtual_keycode, state) {
                    		(Some(VKC::Escape), Pressed) => *closed = true,
							(Some(VKC::Space), Pressed) => pressed.space = true,
                    		(Some(VKC::Space), Released) => pressed.space = false,

                    		(Some(VKC::W), Pressed) => pressed.w = true,
                    		(Some(VKC::W), Released) => pressed.w = false,
                    		(Some(VKC::S), Pressed) => pressed.s = true,
                    		(Some(VKC::S), Released) => pressed.s = false,
                    		(Some(VKC::A), Pressed) => pressed.a = true,
                    		(Some(VKC::A), Released) => pressed.a = false,
                    		(Some(VKC::D), Pressed) => pressed.d = true,
                    		(Some(VKC::D), Released) => pressed.d = false,
                    		_ => (),

                    	},

                    },
                    _ => (),
                },
                _ => (),
            }
        });
}

pub trait Graphics {
	type VertexType : Vertex;
	type V : VectorTrait;

	const VERTEX_SHADER_SRC : &'static str;
	const FRAGMENT_SHADER_SRC : &'static str;

	fn init_glium() -> (winit::EventsLoop,  glium::Display) {
	    let events_loop = glutin::EventsLoop::new();
	    let size = LogicalSize{width : 1024.0,height : 768.0};
	    let wb = glutin::WindowBuilder::new()
	        .with_dimensions(size)
	        .with_title("dim4");;
	    let cb = glutin::ContextBuilder::new();
	    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

	    (events_loop,display)
	}
	fn new() -> Self ;

	fn get_event_loop(&mut self) -> &mut winit::EventsLoop;
	fn get_display(&self) -> &glium::Display;
	fn get_vertex_buffer(&self) -> &glium::VertexBuffer<Self::VertexType>;
    fn get_index_buffer(&self) -> &glium::IndexBuffer<u16>;
    fn get_program(&self) -> &glium::Program;

    fn new_vertex_buffer(&mut self, verts : &Vec<Self::V>);
    fn new_index_buffer(&mut self, verts : &Vec<VertIndex>);

	fn verts_to_gl(verts : &Vec<Self::V>) -> Vec<Self::VertexType>;
	fn vertis_to_gl(vertis : &Vec<VertIndex>) -> Vec<u16> {
    	vertis.iter().map(|v| *v as u16).collect()
	}
	fn draw_lines(&self, verts : &Vec<Self::V>, vertis : &Vec<VertIndex>) {

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

		self.get_vertex_buffer().write(&Self::verts_to_gl(&verts));
        self.get_index_buffer().write(&Self::vertis_to_gl(&vertis));

        let mut target = self.get_display().draw();
        target.clear_color(0.0,0.1,0.1,1.0);
        target.draw(self.get_vertex_buffer(), self.get_index_buffer(), self.get_program(), &uniforms,
            &Default::default()).unwrap();

        target.finish().unwrap();

	}
}


pub trait VertexMake {
	type Vertex : Vertex;
	fn to_vertex(self) -> Self::Vertex;
}
impl VertexMake for crate::vector::Vec2 {
	type Vertex = graphics2d::Vertex;
	fn to_vertex(self) -> Self::Vertex {
		Self::Vertex{position : *self.get_arr() as [f32 ; 2]}
	}
}
// impl VertexMake for crate::vector::Vec3 {
// 	type Vertex = graphics3d::Vertex;
// 	fn to_vertex(self) -> Self::Vertex {
// 		Self::Vertex{position : *self.get_arr() as [f32 ; 3]}
// 	}
// }
// pub fn draw_lines<V,U>(display : glium::Display, verts : Vec<V>, vertis : Vec<VertIndex>)
// where V : VectorTrait + VertexMake
// {
// 	let vertex_shader_src = graphics2d::VERTEX_SHADER_SRC;
// 	let fragment_shader_src = graphics2d::FRAGMENT_SHADER_SRC;

// 	let vertices : Vec<V::Vertex> = verts
// 		.iter()
// 		.map(|v| v.to_vertex())
// 		.collect();
// 	let positions = glium::VertexBuffer::new(&display, &vertices).unwrap();
// 	let indices = glium::index::NoIndices(
// 		glium::index::PrimitiveType::TrianglesList
// 		);

// 	let program = glium::Program::from_source(&display,
// 		vertex_shader_src,
// 		fragment_shader_src, None)
// 	.unwrap();
// }