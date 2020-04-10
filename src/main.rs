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
mod game;
mod input;
mod build_level;


use glium::glutin;
use glium::glutin::dpi::LogicalSize;
use glium::Surface;
use glium::glutin::event_loop::EventLoop;
//use glium::glutin::platform::desktop::EventLoopExtDesktop;

//use glium_text_rusttype as glium_text;
use std::time;
use vector::PI;

//NOTES:
// include visual indicator of what direction a collision is in
use crate::graphics::Graphics;
//threading imports
use std::thread;
use std::sync::mpsc;
use std::sync::{Mutex, Arc};
fn main() {
    use crate::input::Input;
    use glutin::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };
    //test_glium_text();
    let (event_loop, display) = init_glium();

    //more window settings
    //let window_context = display.gl_window();
    //let window = window_context.window();
    //display.gl_window().window().set_always_on_top(true);
    //window.set_always_on_top(true);
    //window.set_cursor_grab(false).unwrap();

    let mut input = Input::new();
    //let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        //game.draw_update(&mut graphics);
        //     for received in rx {
        //         println!("Got: {}", received);
        // }
        let mut graphics = crate::graphics::Graphics2d::new(&display);
        let mut game = game::Game::new(game::build_shapes_3d(),&mut graphics);
    });
    //let val = String::from("hi");
    //tx.send(val).unwrap();
    event_loop.run(move |event, _, control_flow| {

        
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        *control_flow = ControlFlow::Poll;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        *control_flow = ControlFlow::Wait;

        input.listen_events(&event);
        
        // if input.pressed.w {
        //     let val = String::from("hi");
        //     tx.send(val).unwrap();
        // }
        
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

                // Queue a RedrawRequested event.
                //window.request_redraw();
            },
            Event::RedrawRequested(_) => {
                // Redraw the application.
                //
                // It's preferrable to render in this event rather than in MainEventsCleared, since
                // rendering in here allows the program to gracefully handle redraws requested
                // by the OS.
                
            },
            _ => ()
        }
    });

}

fn init_glium() -> (EventLoop<()>,  glium::Display) {
        use glutin::{window::WindowBuilder};
        //use glutin::window::WindowBuilder;
        let event_loop = glutin::event_loop::EventLoop::new();
        let size = LogicalSize{width : 1024.0,height : 768.0};
        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(size)
            .with_title("dim4");

        let cb = glutin::ContextBuilder::new();
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

            //fullscreen
            //window.set_fullscreen(Some(window.get_current_monitor()));
        //let window = WindowBuilder::new().build(&event_loop).unwrap();
        (event_loop,display)
    }


