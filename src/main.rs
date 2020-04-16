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
mod camera;
mod colors;
mod graphics;
mod game;
mod engine;
mod input;
mod build_level;
mod object;


use glium::glutin;
use glium::glutin::dpi::LogicalSize;

use glium::glutin::event_loop::EventLoop;
use glium::Display;

use std::time;

//NOTES:
// include visual indicator of what direction a collision is in

use crate::input::Input;
use engine::Engine;


//threading imports
//use std::thread;
//use std::sync::mpsc;
//use std::sync::{Mutex, Arc};

fn main() {
    
    use glutin::{
        event::{Event, WindowEvent},
        event_loop::ControlFlow,
    };

    let (event_loop, display) = init_glium();
    let mut input = Input::new();

    let mut dim = 3;
    let mut engine = Engine::init(dim,&display);

    let start_instant = time::Instant::now();
    let mut last_instant = time::Instant::now();
    let mut game_duration = time::Instant::now().duration_since(start_instant);
    let mut frame_duration = time::Instant::now().duration_since(last_instant);

    input.closed = false;
    
    //POINT OF NO RETURN. Thanks winit
    event_loop.run(move |event, _, control_flow| {
        
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        *control_flow = ControlFlow::Poll;

        //could use for menus??
        //*control_flow = ControlFlow::Wait;

        //swap / reset engine
        if input.swap_engine {
            dim = match dim {
                3 => Ok(4), 4 => Ok(3), _ => Err("Invalid dimension") 
            }.unwrap();
            engine = Engine::init(dim,&display);
            input.swap_engine = false;
        }
        //input events
        input.listen_events(&event);
        if input.closed {
            println!("Escape button pressed; exiting.");
            *control_flow = ControlFlow::Exit;
        }
        //window / game / redraw events
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            },
            Event::MainEventsCleared => {
                // Application update code.
                game_duration = time::Instant::now().duration_since(start_instant);
                frame_duration = time::Instant::now().duration_since(last_instant);
                last_instant = time::Instant::now();
                // Queue a RedrawRequested event.
                engine.game_update(&mut input, &frame_duration);
                if input.update {
                    display.gl_window().window().request_redraw();
                }
            },
            Event::RedrawRequested(_) => {
                // Redraw the application.
                engine.draw(&display);
                engine.print_debug(&mut input, &frame_duration); 
            },
            _ => ()
        }
    });
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


