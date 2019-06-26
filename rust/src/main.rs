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
    game_3d();
    //buildshapes::build_duoprism_4d([1.0,1.0],[[0,1],[2,3]],[4,4]); 
    //vector::test_vectors();
}
//use crate::vector::{VectorTrait,MatrixTrait};
use crate::graphics::Graphics;
//use draw;
use crate::geometry::{Shape,Line,buildshapes,buildfloor};
use crate::vector::{Vec3,Vec4,VectorTrait};
use crate::input::Input;
use crate::colors::*;
use crate::draw::Camera;

use glium::glutin;
use glium::glutin::dpi::LogicalSize;

use std::time;
use vector::PI;

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

pub fn build_shapes_3d() -> Vec<Shape<Vec3>> {

    //buildshapes::cubeidor_3d()
    let mut tube = buildshapes::build_long_cube_3d(4.0,1.0).set_pos(&Vec3::new(0.5,0.0,0.0));
    let mut tube2 = tube.clone().set_pos(&Vec3::new(3.0,0.0,2.5));

    //let mut tube = buildshapes::invert_normals(&tube);
    //let mut tube2 = buildshapes::invert_normals(&tube2);
    tube2.rotate(0,-1,PI/2.0);

    let cube = buildshapes::build_cube_3d(1.0).set_pos(&Vec3::new(0.5,0.0,2.5));
    //let cube = buildshapes::invert_normals(&cube);

    tube.set_color(RED);
    tube2.set_color(GREEN);
    vec![tube,tube2,cube]
    //vec![cube.clone(),cube.set_pos(&Vec3::new(3.0,0.0,0.0))]
}
pub fn build_shapes_4d() -> Vec<Shape<Vec4>> {
    
    //buildshapes::build_axes_cubes_4d()
    buildshapes::cubeidor_4d()
    //let (m,n) = (4,4);
    //let mut duocylinder = buildshapes::build_duoprism_4d([1.0,1.0],[[0,1],[2,3]],[m,n])

     //   .set_pos(&Vec4::new(0.0,0.0,0.0,0.0));
    
}

pub fn color_duocylinder(shape : &mut Shape<Vec4>, m : usize, n : usize) {
    for (i, face) in itertools::enumerate(shape.faces.iter_mut()) {
        let iint = i as i32;
        face.color = Color([((iint%(m as i32)) as f32)/(m as f32),(i as f32)/((m+n) as f32),1.0,1.0]);
    }
}
pub fn game_3d() {
    
    let (events_loop, display) = init_glium();

    let input = Input::new(events_loop);

    let graphics = crate::graphics::Graphics2d::new(display);
    let shapes = build_shapes_3d();
    let extra_lines = buildfloor::build_floor3(5,1.0,-0.51);
    let mut camera = Camera::new(Vec3::new(2.0,0.0, -10.0));

    camera.look_at(shapes[0].get_pos());
    game(graphics,input,shapes,camera,extra_lines);
}
pub fn game_4d() {
    let (events_loop, display) = init_glium();

    let input = Input::new(events_loop);
    let graphics = crate::graphics::Graphics3d::new(display);
    let shapes = build_shapes_4d();
    for v in &shapes[0].verts {
        println!("{}",v)
    }
    let extra_lines : Vec<Line<Vec4>> = Vec::new();
    let mut camera = Camera::new(Vec4::new(0.0,0.0,0.0, -5.0));
    camera.look_at(shapes[0].get_pos());
    game(graphics,input,shapes,camera,extra_lines);
}
pub fn game<G,V : VectorTrait>(mut graphics : G, mut input : Input,
    mut shapes : Vec<Shape<V>>, mut camera : Camera<V>, extra_lines : Vec<Line<V>>)
where G : Graphics<V::SubV>
{
    // let test_cube = buildshapes::build_cube_3d(1.0)
    //     .set_pos(&Vec3::new(0.0,0.0,0.0));
    //let face_scales = vec![0.1,0.3,0.5,0.7,1.0];
    let face_scales = vec![0.3,0.5,0.8,1.0];

    draw::update_shape_visibility(&camera,&mut shapes);
    let mut draw_lines = draw::transform_draw_lines(
    {
        let mut lines = draw::calc_shapes_lines(&mut shapes,&face_scales,camera.clipping);
        lines.append(&mut crate::draw::calc_lines_color_from_ref(
            &shapes,
            &extra_lines,GRAY));
        lines
    }, &camera);
    //draw_lines.append(&mut crate::draw::draw_wireframe(&test_cube,GREEN));
    let mut cur_lines_length = draw_lines.len();
    
    graphics.new_vertex_buffer_from_lines(&draw_lines);

    let start_instant = time::Instant::now();
    let mut last_instant = time::Instant::now();
    let mut game_duration : time::Duration;
    let mut frame_duration : time::Duration;
    while !input.closed {

        if input.update {
            //if input.pressed.being_touched {
            if false {
                for shape in &mut shapes {
                    shape.rotate(0,-1,0.01)
                }
                //shapes[0].rotate(-2,-1,0.01f32);
                //shapes[1].rotate(-2,-1,0.01f32);
                //hapes[1].rotate(0,1,0.02f32);
            }

            draw::update_shape_visibility(&camera,&mut shapes);
            //draw_lines.append(&mut crate::draw::draw_wireframe(&test_cube,GREEN));
            draw_lines = draw::transform_draw_lines(
            {
                let mut lines = draw::calc_shapes_lines(&mut shapes,&face_scales,camera.clipping);
                lines.append(&mut crate::draw::calc_lines_color_from_ref(
                    &shapes,
                    &extra_lines,CYAN));
                lines
            }, &camera);

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
        game_duration = time::Instant::now().duration_since(start_instant);
        frame_duration = time::Instant::now().duration_since(last_instant);
        last_instant = time::Instant::now();
        input.update_camera(&mut camera, &frame_duration);
        input.update_shape(&mut shapes[1]);
        
        input.print_debug(&camera,&game_duration,&frame_duration);

        
    }
}
