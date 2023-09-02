#[macro_use]
extern crate glium;
#[macro_use]
extern crate itertools;

use config::LevelConfig;
use glium::glutin;
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};

use engine::Engine;
use fps::FPSTimer;
use input::custom_events::CustomEvent;
use winit::window::WindowBuilder;

use crate::{
    config::load_config, input::saveload_dialog::set_load_file_in_config, utils::ValidDimension,
};

mod camera;
mod constants;
mod draw;
mod geometry;
mod vector;

mod build_level;
mod cleanup;
mod coin;
mod collide;
mod components;
mod config;
mod debug;
mod ecs_utils;
mod engine;
mod fps;
mod graphics;
mod gravity;
mod gui;
mod input;
mod player;
mod saveload;
mod shape_entity_builder;
mod spatial_hash;
mod systems;
#[cfg(test)]
mod tests;
mod utils;

//mod object;

//NOTES:
// include visual indicator of what direction a collision is in

fn main() {
    let mut config = load_config();

    let (event_loop, window_builder) = init_winit();
    let display = init_glium(window_builder, &event_loop);

    let mut dim = ValidDimension::Three;
    let mut engine = Engine::new(dim, &config, &display);

    let mut fps_timer = FPSTimer::new();

    display.gl_window().window().set_cursor_visible(false);

    let event_loop_proxy = event_loop.create_proxy();

    //POINT OF NO RETURN. Thanks winit
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        engine.update(&event, &event_loop_proxy, &display, &mut fps_timer);

        if let Event::UserEvent(custom_event) = event {
            match custom_event {
                CustomEvent::SwapEngine => {
                    dim = match dim {
                        ValidDimension::Three => ValidDimension::Four,
                        ValidDimension::Four => ValidDimension::Three,
                    };
                    engine = engine.restart(dim, &config, &display);
                }
                CustomEvent::NewLevel => {
                    config.scene.level = LevelConfig::Empty;
                    engine = engine.restart(dim, &config, &display);
                }
                CustomEvent::RestartLevel => engine = engine.restart(dim, &config, &display),
                CustomEvent::LoadLevel(ref file) => {
                    match set_load_file_in_config(&mut config, file) {
                        Ok(vd) => {
                            dim = vd;
                            engine = engine.restart(dim, &config, &display);
                        }
                        Err(msg) => println!("{}", msg),
                    }
                }
                CustomEvent::Quit => {
                    println!("Exiting.");
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            }
        }
    }); //end of event loop
}

fn init_winit() -> (EventLoop<CustomEvent>, WindowBuilder) {
    let event_loop = winit::event_loop::EventLoopBuilder::<CustomEvent>::with_user_event().build();
    let size = LogicalSize {
        width: 1024.0,
        height: 768.0,
    };
    let wb = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .with_title("dim4");

    (event_loop, wb)
}

fn init_glium(
    window_builder: WindowBuilder,
    event_loop: &EventLoop<CustomEvent>,
) -> glium::Display {
    let cb = glutin::ContextBuilder::new();
    glium::Display::new(window_builder, cb, event_loop).unwrap()

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
}
