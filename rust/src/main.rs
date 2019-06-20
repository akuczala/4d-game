#[macro_use]
extern crate glium;
#[macro_use] extern crate itertools;

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
    //test_glium_2();
    buildshapes::build_duoprism_4d([2.0,3.0],[[0,1],[2,3]],[3,4]); 
    //vector::test_vectors();
    //graphics::graphics3d::test_glium_3d();
}
//use crate::vector::{VectorTrait,MatrixTrait};
use crate::graphics::Graphics;
use crate::geometry::{Shape,buildshapes,buildfloor};
use crate::input::Input;
use crate::colors::*;

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

use crate::vector::Vec3;
pub fn build_shapes_3d() -> Vec<Shape<Vec3>> {

    let mut cube = buildshapes::build_cube_3d(1.0);
    let face_colors = vec![RED,GREEN,BLUE,CYAN,MAGENTA,YELLOW];
    for (face, color) in cube.faces.iter_mut().zip(face_colors) {
        face.color = color;
    }
    let cylinder = buildshapes::build_prism_3d(1.0,1.0,8)
        .set_pos(&Vec3::new(2.0,0.0,0.0));;

    let prism = buildshapes::build_prism_3d(1.0,1.0,3)
        .set_pos(&Vec3::new(0.0,0.0,3.0));
    vec![cube,cylinder,prism]
}

pub fn test_glium_2() {
    use crate::vector::{Vec3};
    use crate::draw::Camera;

    let (events_loop, display) = init_glium();

    let mut input = Input::new(events_loop);
    let mut graphics =  crate::graphics::Graphics2d::new(display);

    let mut shapes = build_shapes_3d();

    let mut camera = Camera::new(Vec3::new(2.0,0.0, -10.0));
    camera.look_at(shapes[0].get_pos());

    let face_scales = vec![0.1,0.3,0.5,0.7,1.0];

    let mut draw_lines = crate::draw::draw_shapes(&camera,&mut shapes,&face_scales);
    draw_lines.append(&mut crate::draw::draw_lines_color(
        &camera, &shapes,
        buildfloor::build_floor3(5,1.0,-1.0),CYAN));
    let mut cur_lines_length = draw_lines.len();
    
    graphics.new_vertex_buffer_from_lines(&draw_lines);

    while !input.closed {

        if input.update {
            if input.pressed.being_touched {
                shapes[1].rotate(1,2,0.001f32);
            }

            draw_lines = crate::draw::draw_shapes(&camera,&mut shapes,&face_scales);
            draw_lines.append(&mut crate::draw::draw_lines_color(
            &camera, &shapes,
            buildfloor::build_floor3(5,1.0,-1.0),CYAN));
            //make new buffer if the number of lines changes
            if draw_lines.len() != cur_lines_length {
                graphics.new_vertex_buffer_from_lines(&draw_lines);
                println!("New buffer! {} to {}",draw_lines.len(),cur_lines_length);
                cur_lines_length = draw_lines.len();
            }
            graphics.draw_lines(&draw_lines);

            
            input.update = true; //set to true for constant updating
        }
        
        input.listen_events();
        input.update_camera(&mut camera);
        input.update_shape(&mut shapes[1]);
        
        input.print_debug(&camera);

        
    }
}
