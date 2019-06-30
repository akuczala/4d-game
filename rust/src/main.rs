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
mod build_level;
//mod text_wrapper;

fn main() {
    let (events_loop, display) = init_glium();

    let mut input = Input::new(events_loop);

    loop {
        game_3d(&mut input,&display);
        if input.closed {
            break
        }
        input.swap_engine = false;
        game_4d(&mut input,&display);
        if input.closed {
            break
        }
        input.swap_engine = false;
    }


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
            .with_title("dim4");

        let cb = glutin::ContextBuilder::new();
        let display = glium::Display::new(wb, cb, &events_loop).unwrap();

        //more window settings
        {
            let window_context = display.gl_window();
            let window = window_context.window();
            //display.gl_window().window().set_always_on_top(true);
            window.set_always_on_top(true);
            window.grab_cursor(false);

            //fullscreen
            //window.set_fullscreen(Some(window.get_current_monitor()));
        }
        (events_loop,display)
    }

pub fn build_shapes_3d() -> Vec<Shape<Vec3>> {
    // let cube = buildshapes::build_cube_3d(1.0);
    // let cube_2 = cube.clone().set_pos(&Vec3::new(0.0,0.0,3.0)).stretch(&Vec3::new(1.0,8.0,1.0));
    // let cube_3 = cube.clone().set_pos(&Vec3::new(-2.0,0.0,0.0)).stretch(&Vec3::new(2.0,2.0,2.0));

    // let shapes = vec![cube,cube_2,cube_3];
    // for shape in &shapes {
    //     println!("radius:{}", shape.radius);
    // }
    //shapes

    build_level::build_lvl_1_3d()
}

pub fn build_shapes_4d() -> Vec<Shape<Vec4>> {
    let wall_length = 3.0;
    //buildshapes::build_axes_cubes_4d()
    //buildshapes::cubeidor_4d()
    let mut shapes = build_level::build_corridor_cross(
        &buildshapes::color_cube(buildshapes::build_cube_4d(1.0)),wall_length);
    //let (m,n) = (4,4);
    //let mut duocylinder = buildshapes::build_duoprism_4d([1.0,1.0],[[0,1],[2,3]],[m,n])
    shapes.push(buildshapes::build_duoprism_4d([0.1,0.1],[[0,1],[2,3]],[6,6])
        .set_color(YELLOW)
        .set_pos(&Vec4::new(0.0,0.0,wall_length - 0.5,0.0)));
    shapes
     //   .set_pos(&Vec4::new(0.0,0.0,0.0,0.0));
    
}

pub fn color_duocylinder(shape : &mut Shape<Vec4>, m : usize, n : usize) {
    for (i, face) in itertools::enumerate(shape.faces.iter_mut()) {
        let iint = i as i32;
        face.color = Color([((iint%(m as i32)) as f32)/(m as f32),(i as f32)/((m+n) as f32),1.0,1.0]);
    }
}
pub fn game_3d(input : &mut Input, display : & glium::Display) {
    
    let graphics = crate::graphics::Graphics2d::new(display);
    let shapes = build_shapes_3d();
    //let mut extra_lines = buildfloor::build_floor3(5,1.0,0.0);
    //extra_lines.append(&mut buildfloor::build_floor3(5,1.0,1.0));
    let extra_lines : Vec<Line<Vec3>> = Vec::new();
    let mut camera = Camera::new(Vec3::new(0.0,0.0,0.0));

    //camera.look_at(shapes[0].get_pos());
    game(graphics,input,shapes,camera,extra_lines);
}
pub fn game_4d(input : &mut Input, display : & glium::Display) {

    let graphics = crate::graphics::Graphics3d::new(display);
    let shapes = build_shapes_4d();

    let extra_lines : Vec<Line<Vec4>> = Vec::new();
    let mut camera = Camera::new(Vec4::new(0.0,0.0,0.0,0.0));
    //camera.look_at(shapes[0].get_pos());
    game(graphics,input,shapes,camera,extra_lines);
}
pub fn game<'a,G,V : VectorTrait>(mut graphics : G,input : &mut Input,
    mut shapes : Vec<Shape<V>>, mut camera : Camera<V>, extra_lines : Vec<Line<V>>)
where G : Graphics<'a,V::SubV>
{
    let mut in_front = clipping::init_in_front(&shapes);
    // let test_cube = buildshapes::build_cube_3d(1.0)
    //     .set_pos(&Vec3::new(0.0,0.0,0.0));
    //let face_scales = vec![0.1,0.3,0.5,0.7,1.0];
    //let face_scales = vec![0.3,0.5,0.8,1.0];
    let face_scales = vec![0.5,0.99];
    draw::update_shape_visibility(&camera,&mut shapes);
    clipping::calc_in_front(&mut in_front, &shapes,&camera.pos);

    let mut draw_lines = draw::transform_draw_lines(
    {
        let mut lines = draw::calc_shapes_lines(&mut shapes,&face_scales,camera.clipping, &in_front);
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
    input.closed = false;
    while !input.closed && !input.swap_engine {

        if input.update {
            //if input.pressed.being_touched {
            if true {
                let shapes_len = shapes.len();
                shapes[shapes_len-1].rotate(0,-1,0.05);
                // for shape in &mut shapes {
                //     shape.rotate(0,-1,0.01)
                // }
                //shapes[0].rotate(-2,-1,0.01f32);
                //shapes[1].rotate(-2,-1,0.01f32);
                //hapes[1].rotate(0,1,0.02f32);
            }

            draw::update_shape_visibility(&camera,&mut shapes);
            clipping::calc_in_front(&mut in_front, &shapes,&camera.pos);
            //draw_lines.append(&mut crate::draw::draw_wireframe(&test_cube,GREEN));
            draw_lines = draw::transform_draw_lines(
            {
                let mut lines = draw::calc_shapes_lines(&mut shapes,&face_scales,camera.clipping, &in_front);
                lines.append(&mut crate::draw::calc_lines_color_from_ref(
                    &shapes,
                    &extra_lines,CYAN));
                lines
            }, &camera);

            //make new buffer if the number of lines changes
            if draw_lines.len() != cur_lines_length {
                graphics.new_vertex_buffer_from_lines(&draw_lines);
                //println!("New buffer! {} to {}",draw_lines.len(),cur_lines_length);
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
        
        input.print_debug(&camera,&game_duration,&frame_duration,&in_front,&shapes);

        
    }
}
