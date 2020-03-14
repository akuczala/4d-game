#[macro_use]
extern crate glium;
#[macro_use] extern crate itertools;

#[allow(dead_code)]
mod vector;
#[allow(dead_code)]
mod geometry;
#[allow(dead_code)]
mod clipping;
#[allow(dead_code)]
mod draw;
#[allow(dead_code)]
mod colors;
mod graphics;
mod input;
mod build_level;

//mod text_wrapper;

// fn test_freetype() {
//     use freetype::Library;

//     // Init the library
//     let lib = Library::init().unwrap();
//     // Load a font face
//     let face = lib.new_face("arial.ttf", 0).unwrap();
//     // Set the font size
//     face.set_char_size(40 * 64, 0, 50, 0).unwrap();
//     // Load a character
//     face.load_char('A' as usize, freetype::face::RENDER).unwrap();
//     // Get the glyph instance
//     let glyph = face.glyph();
//     do_something_with_bitmap(glyph.bitmap());
// }
//use glium_text_rusttype as glium_text;
// fn test_glium_text() {
//     //use glium::Display::backend::Facade;    
//     let (events_loop, display) = init_glium();
//     // The `TextSystem` contains the shaders and elements used for text display.
//     let system = glium_text::TextSystem::new(&display);

//     // Creating a `FontTexture`, which a regular `Texture` which contains the font.
//     // Note that loading the systems fonts is not covered by this library.
//     let font = glium_text::FontTexture::new(&display, std::fs::File::open(&std::path::Path::new("arial.ttf")).unwrap(), 24).unwrap();

//     // Creating a `TextDisplay` which contains the elements required to draw a specific sentence.
//     let text = glium_text::TextDisplay::new(&system, &font, "Hello world!");

//     // Finally, drawing the text is done like this:
//     let matrix = [[1.0, 0.0, 0.0, 0.0],
//               [0.0, 1.0, 0.0, 0.0],
//               [0.0, 0.0, 1.0, 0.0],
//               [0.0, 0.0, 0.0, 1.0]];
//     glium_text::draw(&text, &system, &mut display.draw(), matrix, (1.0, 1.0, 0.0, 1.0));
//     loop{}
// }


//NOTES:
// include visual indicator of what direction a collision is in
// 'fog' at large distances: fade out in alpha
// point clouds as textures
//use itertools method set_from to streamline modifying mutable vecs
fn main() {

    //test_glium_text();

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
use crate::graphics::Graphics;

use crate::geometry::{Shape,Line,buildshapes};
use crate::vector::{Vec3,Vec4,VectorTrait};
use crate::input::Input;
use crate::colors::*;
use crate::draw::{Camera,Buffer,DrawLine};


use glium::glutin;
use glium::glutin::dpi::LogicalSize;


//use glium_text_rusttype as glium_text;
use std::time;
use vector::PI;

fn init_glium() -> (glium::glutin::EventsLoop,  glium::Display) {
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
            window.grab_cursor(false).unwrap();

            //fullscreen
            //window.set_fullscreen(Some(window.get_current_monitor()));
        }
        (events_loop,display)
    }

pub fn build_shapes_3d() -> Vec<Shape<Vec3>> {

    build_level::build_lvl_1_3d(&vec![0.7,0.9])
    //build_level::build_test_scene_3d()
}

pub fn build_shapes_4d() -> Vec<Shape<Vec4>> {
    let wall_length = 3.0;
    //buildshapes::build_axes_cubes_4d()
    //buildshapes::cubeidor_4d()
    let mut shapes = build_level::build_corridor_cross(
        &buildshapes::color_cube(buildshapes::build_cube_4d(1.0)),wall_length,&vec![0.7,0.9]);
    //let (m,n) = (4,4);
    //let mut duocylinder = buildshapes::build_duoprism_4d([1.0,1.0],[[0,1],[2,3]],[m,n])
    shapes.push(buildshapes::build_duoprism_4d([0.1,0.1],[[0,1],[2,3]],[6,6])
        .set_color(YELLOW)
        .set_pos(&Vec4::new(0.0,0.0,wall_length - 0.5,0.0)));
    //let shapes_len = shapes.len();
    //buildshapes::color_duocylinder(&mut shapes[shapes_len-1],10,10);
    shapes
     //   .set_pos(&Vec4::new(0.0,0.0,0.0,0.0));
    
}

pub fn game_3d(input : &mut Input, display : & glium::Display) {
    
    let graphics = crate::graphics::Graphics2d::new(display);
    let shapes = build_shapes_3d();
    //let mut extra_lines = buildfloor::build_floor3(5,1.0,0.0);
    //extra_lines.append(&mut buildfloor::build_floor3(5,1.0,1.0));
    let extra_lines : Vec<Line<Vec3>> = Vec::new();
    //let extra_lines = draw::Texture::make_tile_texture(&vec![0.3,0.8],&vec![3,4,5]);
    let camera = Camera::new(Vec3::new(0.0,0.0,0.0));

    //camera.look_at(shapes[0].get_pos());
    game(graphics,input,shapes,camera,extra_lines);
}
pub fn game_4d(input : &mut Input, display : & glium::Display) {

    let graphics = crate::graphics::Graphics3d::new(display);
    let shapes = build_shapes_4d();

    let extra_lines : Vec<Line<Vec4>> = Vec::new();
    let camera = Camera::new(Vec4::new(0.0,0.0,0.0,0.0));
    //camera.look_at(shapes[0].get_pos());
    game(graphics,input,shapes,camera,extra_lines);
}
pub fn game<'a,G,V : VectorTrait>(mut graphics : G,input : &mut Input,
    mut shapes : Vec<Shape<V>>, mut camera : Camera<V>, extra_lines : Vec<Line<V>>)
where G : Graphics<'a,V::SubV>
{


    fn draw_stuff<V : VectorTrait>(
        line_buffer : &mut Buffer<Option<DrawLine<V>>>,
        proj_line_buffer : &mut Buffer<Option<DrawLine<V::SubV>>>,
        shape_buffer : &mut Buffer<Option<DrawLine<V>>>,
        extra_buffer : &mut Buffer<Option<DrawLine<V>>>,
        
        camera : &Camera<V>,
        shapes : &mut Vec<Shape<V>>,
        clip_state : &mut clipping::ClipState<V>,
        extra_lines : &Vec<Line<V>>
    ) {
        
        //let face_scales = vec![0.1,0.3,0.5,0.7,1.0];
        //let face_scales = vec![0.3,0.5,0.8,1.0];
        let face_scales = vec![0.7,0.9];

        draw::update_shape_visibility(&camera, shapes, clip_state);
        clip_state.calc_in_front(&shapes,&camera.pos);

        shape_buffer.clear();
        proj_line_buffer.clear();        

        draw::calc_shapes_lines(line_buffer,shape_buffer,extra_buffer,shapes,&face_scales,&clip_state);
        draw::transform_draw_lines(line_buffer,proj_line_buffer,&camera);
        // draw::transform_draw_lines(
        // {
        //     let mut lines = draw::calc_shapes_lines(shapes,&face_scales,&clip_state);
        //     lines.append(&mut crate::draw::calc_lines_color_from_ref(
        //         &shapes,
        //         &extra_lines,CYAN));
        //     lines
        // }, &camera)
    }
    let mut line_buffer : Buffer<Option<DrawLine<V>>> = Buffer::new();
    let mut shape_buffer : Buffer<Option<DrawLine<V>>> = Buffer::new();
    let mut extra_buffer : Buffer<Option<DrawLine<V>>> = Buffer::new();
    let mut proj_line_buffer : Buffer<Option<DrawLine<V::SubV>>> = Buffer::new();

    let mut clip_state = clipping::ClipState::new(&shapes);
    // let test_cube = buildshapes::build_cube_3d(1.0)
    //     .set_pos(&Vec3::new(0.0,0.0,0.0));

    
    draw_stuff(&mut line_buffer, &mut proj_line_buffer, &mut shape_buffer, &mut extra_buffer,
        &camera, &mut shapes, &mut clip_state, &extra_lines);
    //draw_lines.append(&mut crate::draw::draw_wireframe(&test_cube,GREEN));
    let mut cur_lines_length = proj_line_buffer.cur_size;
    
    println!("{}",line_buffer.cur_size);
    println!("{}",extra_buffer.cur_size);
    println!("{}",proj_line_buffer.cur_size);

    graphics.new_vertex_buffer_from_lines(proj_line_buffer.get_slice());

    let start_instant = time::Instant::now();
    let mut last_instant = time::Instant::now();
    let mut game_duration : time::Duration;
    let mut frame_duration : time::Duration;
    input.closed = false;
    while !input.closed && !input.swap_engine {

        if input.update {
            //if input.pressed.being_touched {
            if false {
                let shapes_len = shapes.len();
                shapes[shapes_len-1].rotate(0,-1,0.05);

            }

            draw_stuff(&mut line_buffer, &mut proj_line_buffer, &mut shape_buffer, &mut extra_buffer,
        &camera, &mut shapes, &mut clip_state, &extra_lines);

            //make new buffer if the number of lines changes
            if proj_line_buffer.cur_size != cur_lines_length {
            //if true {
                graphics.new_vertex_buffer_from_lines(proj_line_buffer.get_slice());
                //println!("New buffer! {} to {}",draw_lines.len(),cur_lines_length);
                cur_lines_length = proj_line_buffer.cur_size;
            }
            graphics.draw_lines(proj_line_buffer.get_slice());

            
            input.update = true; //set to true for constant updating
        }
        
        input.listen_events();
        game_duration = time::Instant::now().duration_since(start_instant);
        frame_duration = time::Instant::now().duration_since(last_instant);
        last_instant = time::Instant::now();
        input.update_camera(&mut camera, &frame_duration);
        let shapes_len = shapes.len();
        input.update_shape(&mut shapes[shapes_len-1]);
        
        input.print_debug(&camera,&game_duration,&frame_duration,&mut clip_state,&shapes);

        
    }
}
