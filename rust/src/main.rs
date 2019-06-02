#[macro_use]
extern crate glium;

#[allow(dead_code)]
mod vector;
#[allow(dead_code)]
mod geometry;
mod clipping;
#[allow(dead_code)]
mod draw;
#[allow(dead_code)]
mod colors;
mod graphics;
mod input;
//mod text_wrapper;

fn main() {
    test_glium_2();
    //graphics::graphics3d::test_glium_3d();
}
use crate::vector::{VectorTrait,MatrixTrait};
use crate::graphics::Graphics;
use crate::input::Input;
use glium::glutin;
use glium::glutin::dpi::LogicalSize;

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

pub fn test_glium_2() {
    use crate::vector::{Vec3};
    use crate::draw::Camera;

    let (events_loop, display) = init_glium();

    let mut input = Input::new(events_loop);
    let mut graphics =  crate::graphics::Graphics2d::new(display);

    let mut cylinder = crate::geometry::buildshapes::build_cylinder(1.0,2.0,10);
    let mut camera = Camera::new(Vec3::new(0.0, 0.0, -10.0));
    

    //let lines = crate::draw::draw_wireframe(&camera,&cylinder);
    let lines = crate::draw::draw_shape(&camera,&cylinder);
    //vertex buffer (and presumably index buffer) do not allow size of array
    //to change (at least using the write operation)
    
    graphics.new_vertex_buffer_from_lines(&lines);
    //graphics.new_index_buffer(&vertis);

    //let mut t: f32 = -0.5;
    let mut update = true;
    while !input.closed {

        if update {
            //cylinder.rotate(1,2,0.001f32);
            //let lines = crate::draw::draw_wireframe(&camera,&cylinder);
            cylinder.update_visibility(camera.pos);
            let lines = crate::draw::draw_shape(&camera,&cylinder);
            graphics.draw_lines(&lines);

            
            update = false;
        }
        
        input.listen_events();
        update = input.update_camera(&mut camera);
        input.print_debug(&camera);

        
    }
}
