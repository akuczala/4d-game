use crate::engine::Player;
use std::marker::PhantomData;

use glium::glutin;
use glutin::event::VirtualKeyCode as VKC;

use winit_input_helper::WinitInputHelper;

use specs::prelude::*;

use std::time::Duration;

use crate::camera::Camera;
use crate::vector::{VectorTrait,Field,VecIndex};
use crate::geometry::Shape;
use crate::clipping::ClipState;
use crate::collide::MoveNext;

//use crate::game::Game;

use glutin::event::{Event,WindowEvent};

fn duration_as_field(duration : &Duration) -> f32 {
 (duration.as_secs() as Field) + 0.001*(duration.subsec_millis() as Field)
}

pub struct Input {
    pub helper : WinitInputHelper,
    //pub pressed : ButtonsPressed,
    pub closed : bool,
    pub swap_engine : bool,
    pub update : bool,
    pub frame_duration : crate::fps::FPSFloat,
    pub mouse_dpos : (f32,f32),
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
            mouse_dpos : (0.,0.),
            movement_mode : MovementMode::Mouse,
        }
    }
}

pub struct UpdateCameraSystem<V : VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for UpdateCameraSystem<V> {
    type SystemData = (Write<'a,Input>,WriteStorage<'a,Camera<V>>,WriteStorage<'a,MoveNext<V>>,ReadExpect<'a,Player>);
    fn run(&mut self, (mut input, mut camera, mut move_next, player) : Self::SystemData) {
        update_camera(&mut input, &mut camera.get_mut(player.0).unwrap(), move_next.get_mut(player.0).unwrap());
    }
}

pub enum MovementMode{Tank, Mouse} //add flying tank mode, maybe flying mouse mode

//(- key, + key, axis)
const MOVE_KEYMAP : [(VKC,VKC,VecIndex); 3] = [
    (VKC::A, VKC::D, 0),
    (VKC::K, VKC::I, 1),
    (VKC::Q, VKC::E, 2),
];

const MOUSE_SENSITIVITY : Field = 0.2;
pub fn update_camera<V : VectorTrait>(input : &mut Input, camera : &mut Camera<V>, move_next : &mut MoveNext<V>)
{
    //let frame_time = duration_as_field(frame_duration) as Field;
    let frame_time = input.frame_duration as Field;
    let mut any_slide_turn = false;

    //mouse
    match input.movement_mode {
        MovementMode::Mouse => {
            let mouse_pos = input.helper.mouse();
            let (mx, my) = match mouse_pos {
                Some((x,y)) => (x-100.,y-100.),
                None => (0.,0.)

            };
            //x mouse movement
            let (dmx, dmy) = input.helper.mouse_diff();
            if dmx.abs() != 0. {
                if input.helper.held_shift() {
                    camera.turn(0,2, mx*frame_time*MOUSE_SENSITIVITY);
                } else {
                    camera.turn(0,-1,mx*frame_time*MOUSE_SENSITIVITY);
                }
                
                //camera.spin(0,-1,mx*frame_time*MOUSE_SENSITIVITY);
                any_slide_turn = true;
            }
            //y mouse movement
            if dmy.abs() != 0. {

                match (V::DIM, input.helper.held_shift()) {
                    (3, _) | (4, true) => camera.tilt(1,-1,-my*frame_time*MOUSE_SENSITIVITY),
                    (4, false) => camera.turn(2,-1,-my*frame_time*MOUSE_SENSITIVITY),
                    (_, _) => panic!("Invalid dimension"),
                };
                //camera.spin(axis,-1,-my*frame_time*MOUSE_SENSITIVITY);
                any_slide_turn = true;
            }
        },
        MovementMode::Tank => (),
    }

    //keyboard

    //fowards + backwards
    if input.helper.key_held(VKC::W) {
        *move_next = MoveNext{
            next_dpos : Some(camera.get_slide_dpos(camera.heading[-1],frame_time)),
            can_move : Some(true)
        };
        //camera.slide(camera.heading,frame_time);
        input.update = true;
    }
    if input.helper.key_held(VKC::S) {
        *move_next = MoveNext{
            next_dpos : Some(camera.get_slide_dpos(-camera.heading[-1],frame_time)),
            can_move : Some(true)
        };
        input.update = true;
    }

    //sliding,turning
    for &(key_minus, key_plus, axis) in MOVE_KEYMAP.iter() {

        let movement_sign = 
            input.helper.key_held(key_plus) as i32 -
            input.helper.key_held(key_minus) as i32;
        let movement_sign = movement_sign as f32;

        if movement_sign != 0. {
            any_slide_turn = true;
            //sliding
            if input.helper.held_alt() ^ match input.movement_mode {MovementMode::Mouse => true, _ => false} {
                *move_next = MoveNext{
                    next_dpos : Some(camera.get_slide_dpos(camera.heading[axis]*movement_sign,frame_time)),
                    can_move : Some(true)
                };
                //camera.slide(camera.frame[axis]*movement_sign,frame_time)
            //rotations
            } else { 
                //special case : (0,2) rotation
                if V::DIM == 4 && input.helper.held_shift() && axis == 2 {
                    camera.spin(0,2,movement_sign*frame_time)
                //turning: rotation along (axis,-1)
                } else {
                    if axis == 1 {
                        camera.tilt(axis,-1,movement_sign*frame_time);
                    } else {
                        camera.turn(axis,-1,movement_sign*frame_time);
                    }
                    
                    //camera.spin(axis,-1,movement_sign*frame_time)
                }
                
            }
        };

    }
    //spin unless turning or sliding
    if V::DIM == 4 && any_slide_turn == false {
        camera.spin(0,2,0.05*frame_time);
    }
    //         //reset orientation
    //         if !input.pressed.space {
    //             camera.frame = V::M::id();
    //             camera.update();
    //             input.update = true;
    //             input.pressed.space = true;

}
pub fn update_shape<V : VectorTrait>(input : &mut Input, shape : &mut Shape<V>)
{
    //toggle transparency
    if input.helper.key_released(VKC::T) {
        shape.transparent = !shape.transparent;
        //input.pressed.t = true;
        input.update = true;
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
                MovementMode::Mouse => MovementMode::Tank,
                MovementMode::Tank => MovementMode::Mouse,
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
                _ => (),
            },
            _ => (),
        }
        if self.helper.update(&ev) {
            self.listen_inputs();
        }


    }

}
