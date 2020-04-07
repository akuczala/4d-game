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
fn main() {
    use crate::input::Input;
    use glutin::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };
    //test_glium_text();

    let (event_loop, display, window) = init_glium();

    let mut input = Input::new();

    event_loop.run(move |event, _, control_flow| {
    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    *control_flow = ControlFlow::Poll;

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    *control_flow = ControlFlow::Wait;

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
            window.request_redraw();
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

use glium::glutin;
use glium::glutin::dpi::LogicalSize;
use glium::Surface;
use glium::glutin::event_loop::EventLoop;

//use glium_text_rusttype as glium_text;
use std::time;
use vector::PI;

fn init_glium() -> (EventLoop<()>,  glium::Display, glutin::window::Window) {
        use glutin::window::WindowBuilder;
        let event_loop = glutin::event_loop::EventLoop::new();
        let size = LogicalSize{width : 1024.0,height : 768.0};
        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(size)
            .with_title("dim4");

        let cb = glutin::ContextBuilder::new();
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        //more window settings
        {
            let window_context = display.gl_window();
            let window = window_context.window();
            //display.gl_window().window().set_always_on_top(true);
            window.set_always_on_top(true);
            window.set_cursor_grab(false).unwrap();

            //fullscreen
            //window.set_fullscreen(Some(window.get_current_monitor()));
        }
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        (event_loop,display, window)
    }


