#![allow(dead_code)]
#![allow(unused_imports)]
#[macro_use] extern crate glium;
#[macro_use] extern crate itertools;

//use specs::prelude::*;
use glium::glutin;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event_loop::EventLoop;

use engine::Engine;
use fps::FPSTimer;

//extern crate imgui;
//extern crate imgui_glium_renderer;
mod constants;
mod vector;
mod geometry;
mod draw;
mod camera;

mod graphics;
mod gui;
mod engine;
mod shape_entity_builder;
mod collide;
mod gravity;
mod spatial_hash;
mod input;
mod build_level;
mod coin;
mod fps;
mod player;
mod cleanup;
mod systems;
mod components;
mod ecs_utils;
mod tests;
//mod object;

//NOTES:
// include visual indicator of what direction a collision is in

fn main() {
    
    use glutin::{
        //event::{Event, WindowEvent},
        event_loop::ControlFlow,
    };

    let (event_loop, display) = init_glium();

    let mut dim = 3;
    let mut engine = Engine::init(dim,&display);

    let mut fps_timer = FPSTimer::new();

    display.gl_window().window().set_cursor_visible(false);
    
    //POINT OF NO RETURN. Thanks winit
    event_loop.run(move |event, _, control_flow| {
        //let mut engine = tengine.take_mut().unwrap();
        
        //ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        *control_flow = ControlFlow::Poll;

        //could use for menus??
        //*control_flow = ControlFlow::Wait;
        
        let swap = engine.update(&event,control_flow,&display, &mut fps_timer);

        if swap {
            dim = match dim {
                3 => Ok(4), 4 => Ok(3), _ => Err("Invalid dimension") 
            }.unwrap();
            engine = engine.swap_dim(&display);
        }

    }); //end of event loop

}

fn init_glium() -> (EventLoop<()>,  glium::Display) {

    let event_loop = glutin::event_loop::EventLoop::new();
    let size = LogicalSize{width : 1024.0,height : 768.0};
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(size)
        .with_title("dim4");

    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    //borrow issues here
    //more window settings
    //let window_context = display.gl_window();
    // let window = display.gl_window().window();
    // display.gl_window().window().set_always_on_top(true);
    // window.set_always_on_top(true);
    //window.set_cursor_grab(false).unwrap();
    //fullscreen
    //window.set_fullscreen(Some(window.get_current_monitor()));
    //let window = WindowBuilder::new().build(&event_loop).unwrap();

    (event_loop,display)
}


