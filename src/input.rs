mod selection;
mod update_camera;
mod input_to_transform;

pub use selection::*;
pub use update_camera::*;

use crate::player::Player;
use std::marker::PhantomData;

use glium::glutin;
use glium::glutin::dpi::PhysicalPosition;
use glutin::event::VirtualKeyCode as VKC;
use glutin::event::{TouchPhase,MouseScrollDelta};
use glutin::dpi::LogicalPosition;

use winit_input_helper::WinitInputHelper;

use specs::prelude::*;

use crate::vector::{VectorTrait,Field,VecIndex};
use crate::components::*;

use glutin::event::{Event,WindowEvent};
use crate::geometry::shape::RefShapes;

// fn duration_as_field(duration : &Duration) -> f32 {
//  (duration.as_secs() as Field) + 0.001*(duration.subsec_millis() as Field)
// }

//maximum allowed time step (player movement slows down at around 20 FPS)
//this prevents the player from jumping through walls
//we could be a little more careful with the nearest face detection
//right now, if the player ends up inside a wall, the nearest face is misidentified, because it's a signed distance
//see shape.rs
const MAX_DT : Field = 1000./20.;

const MOUSE_SENSITIVITY : Field = 0.2;
const MOUSE_STICK_POINT : [f32 ; 2] = [100.,100.];

#[derive(Copy, Clone,PartialEq,Eq)]
pub enum PlayerMovementMode{
    Tank,
    Mouse
}

#[derive(Copy, Clone,PartialEq,Eq)]
pub enum MovementMode{Player(PlayerMovementMode), Shape(ShapeMovementMode)} //add flying tank mode, maybe flying mouse mode

#[derive(Default)]
pub struct MouseData {
    pub mouse_dpos : (f32,f32),
    pub scroll_dpos: Option<(f32,f32)>,
    pub integrated_mouse_dpos: (f32, f32),
    pub integrated_scroll_dpos: (f32, f32),
}
// TODO: better organized input modes + states
pub struct Input {
    pub helper : WinitInputHelper,
    //pub pressed : ButtonsPressed,
    pub closed : bool,
    pub swap_engine : bool,
    pub update : bool,
    pub frame_duration : crate::fps::FPSFloat,
    pub mouse: MouseData,
    pub movement_mode : MovementMode,
}
impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
impl Input {

    pub fn new() -> Self {
        Input{
            helper : WinitInputHelper::new(),
            //pressed : ButtonsPressed::new(),
            closed : false,
            swap_engine : false,
            update : true,
            frame_duration : crate::fps::TARGET_FPS,
            mouse: Default::default(),
            movement_mode : MovementMode::Player(PlayerMovementMode::Mouse),
        }
    }
}
impl Input {
    pub fn get_dt(&self) -> Field {
        (self.frame_duration as Field).min(MAX_DT)
    }
    pub fn is_camera_movement_enabled(&self) -> bool {
        match self.movement_mode {
            MovementMode::Shape(ShapeMovementMode::Free) => false,
            _ => true
        }
    }
}



pub struct PrintDebugSystem<V : VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for PrintDebugSystem<V> {

    type SystemData = (Write<'a,Input>,Write<'a,ClipState<V>>);

    fn run(&mut self, (mut input, mut clip_state) : Self::SystemData) {
        print_debug::<V>(&mut input,&mut clip_state);
    }
}

pub fn print_debug<V : VectorTrait>(input : &mut Input,clip_state : &mut ClipState<V>)
{
    if input.helper.key_released(VKC::Space) {
        //println!("camera.pos = {}",camera.pos);
        //rintln!("camera.heading = {}",camera.heading);
        //println!("camera.frame = {}",camera.frame);
        //println!("game time elapsed: {}", duration_as_field(game_time));
        //let frame_seconds = duration_as_field(frame_len);
        println!("frame time: {}, fps: {}", input.frame_duration,1.0/input.frame_duration);
        //clipping::print_in_front(&clip_state.in_front);
        //clip_state.print_debug();
        //clipping::test_dyn_separate(&shapes,&camera.pos);
        //input.pressed.space = true;

    }
    //toggle clipping
    if input.helper.key_released(VKC::C) {
        //TEMPORARILY DISABLED
        clip_state.clipping_enabled = !clip_state.clipping_enabled;
        println!("clipping={}",clip_state.clipping_enabled);
        //input.pressed.c = true;
        input.update = true;
    }
}

impl Input {
    pub fn listen_inputs(&mut self) {
        if self.helper.key_released(VKC::Escape) {
            self.closed = true
        }
        if self.helper.key_released(VKC::Back) {
            self.swap_engine = true
        }
        if self.helper.key_released(VKC::M) {
            self.movement_mode = match self.movement_mode {
                MovementMode::Player(player_mode) => MovementMode::Player(
                    match player_mode {
                        PlayerMovementMode::Mouse => PlayerMovementMode::Tank,
                        PlayerMovementMode::Tank => PlayerMovementMode::Mouse
                    }
                ),
                _ => MovementMode::Player(PlayerMovementMode::Mouse)
            }
        }
        for &(key, mode) in MODE_KEYMAP.iter() {
            if self.helper.key_released(key) {
                self.movement_mode = MovementMode::Shape(mode)
            }
        }

    }
    // listing the events produced by application and waiting to be received
    pub fn listen_events<E>(&mut self, ev : &Event<E>) {
        let closed = &mut self.closed;
        let update = &mut self.update;
        //let swap_engine = &mut self.swap_engine;

        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *closed = true,
                WindowEvent::Resized(_) => *update = true,
                WindowEvent::Touch(glutin::event::Touch{phase, ..}) => match phase {
                        glutin::event::TouchPhase::Started => (),
                        glutin::event::TouchPhase::Ended => (),
                        _ => (),

                    }
                e => mouse_event(&mut self.mouse, e),
            },
            _ => (),
        }
        if self.helper.update(ev) {
            self.listen_inputs();
            
        }


    }

}

fn mouse_event(mouse: &mut MouseData, window_event: &WindowEvent) {
    match window_event {
        WindowEvent::CursorMoved{position,..} => {
            mouse.mouse_dpos.0 = position.x as f32 - MOUSE_STICK_POINT[0];
            mouse.mouse_dpos.1 = position.y as f32 - MOUSE_STICK_POINT[1];
        },
        WindowEvent::MouseWheel {delta,phase, ..} => {
            match phase {
                TouchPhase::Started => {},
                TouchPhase::Moved => {
                    mouse.scroll_dpos = match delta {
                        MouseScrollDelta::LineDelta(x,y) => Some((*x,*y)),
                        MouseScrollDelta::PixelDelta(PhysicalPosition{x,y})
                            => Some((*x as f32,*y as f32))
                    }
                },
                TouchPhase::Cancelled | TouchPhase::Ended => {
                    mouse.scroll_dpos = None;
                },
            }
            //println!("{:?}, {:?}",delta, phase)
        },
        _ => (),
    }
}