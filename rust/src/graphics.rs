pub mod graphics2d;
pub mod graphics3d;

use glium::glutin;
use glium::Surface;

//use glium::Surface;
use glium::glutin::dpi::LogicalSize;
use glium::vertex::Vertex;

use crate::vector::{VectorTrait};
use crate::geometry::VertIndex;

fn init_glium() -> (winit::EventsLoop, glium::Display){
    let events_loop = glutin::EventsLoop::new();
    let size = LogicalSize{width : 1024.0,height : 768.0};
    let wb = glutin::WindowBuilder::new()
        .with_dimensions(size)
        .with_title("dim4");;
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    (events_loop,display)
}
struct ButtonsPressed {
	w : bool,
	s : bool,
	a : bool,
	d : bool
}
impl ButtonsPressed {
	fn new() -> Self{
		ButtonsPressed {
			w: false, s: false, a : false, d : false
		}
	}
}
// listing the events produced by application and waiting to be received
fn listen_events(events_loop :  &mut winit::EventsLoop, closed: &mut bool, pressed : &mut ButtonsPressed) {
    events_loop.poll_events(|ev| {
            match ev {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => *closed = true,
                    glutin::WindowEvent::KeyboardInput{input, ..} => match input {
                    	glutin::KeyboardInput{ virtual_keycode, state, ..} => match (virtual_keycode, state) {
                    		(Some(glutin::VirtualKeyCode::W), glutin::ElementState::Pressed) => pressed.w = true,
                    		(Some(glutin::VirtualKeyCode::W), glutin::ElementState::Released) => pressed.w = false,
                    		(Some(glutin::VirtualKeyCode::S), glutin::ElementState::Pressed) => pressed.s = true,
                    		(Some(glutin::VirtualKeyCode::S), glutin::ElementState::Released) => pressed.s = false,
                    		(Some(glutin::VirtualKeyCode::A), glutin::ElementState::Pressed) => pressed.a = true,
                    		(Some(glutin::VirtualKeyCode::A), glutin::ElementState::Released) => pressed.a = false,
                    		(Some(glutin::VirtualKeyCode::D), glutin::ElementState::Pressed) => pressed.d = true,
                    		(Some(glutin::VirtualKeyCode::D), glutin::ElementState::Released) => pressed.d = false,
                    		_ => (),

                    	},

                    },
                    _ => (),
                },
                _ => (),
            }
        });
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
impl VertexMake for crate::vector::Vec3 {
	type Vertex = graphics3d::Vertex;
	fn to_vertex(self) -> Self::Vertex {
		Self::Vertex{position : *self.get_arr() as [f32 ; 3]}
	}
}
pub fn draw_lines<V,U>(display : glium::Display, verts : Vec<V>, vertis : Vec<VertIndex>)
where V : VectorTrait + VertexMake
{
	let vertex_shader_src = graphics2d::VERTEX_SHADER_SRC;
	let fragment_shader_src = graphics2d::FRAGMENT_SHADER_SRC;

	let vertices : Vec<V::Vertex> = verts
		.iter()
		.map(|v| v.to_vertex())
		.collect();
	let positions = glium::VertexBuffer::new(&display, &vertices).unwrap();
	let indices = glium::index::NoIndices(
		glium::index::PrimitiveType::TrianglesList
		);

	let program = glium::Program::from_source(&display,
		vertex_shader_src,
		fragment_shader_src, None)
	.unwrap();
}