use super::key_map::{MOVE_KEYMAP, MOVE_FORWARDS, MOVE_BACKWARDS};
use super::{Input, MovementMode, MOUSE_SENSITIVITY};

use crate::player::Player;
use std::marker::PhantomData;

use glium::glutin;
use glutin::event::VirtualKeyCode as VKC;
use glutin::event::{TouchPhase,MouseScrollDelta};
use glutin::dpi::LogicalPosition;

use winit_input_helper::WinitInputHelper;

use specs::prelude::*;

use crate::vector::{VectorTrait,Field,VecIndex};
use crate::components::*;

use glutin::event::{Event,WindowEvent};
use crate::geometry::shape::RefShapes;
use crate::input::{PlayerMovementMode, ShapeMovementMode};

pub struct UpdateCameraSystem<V : VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for UpdateCameraSystem<V> {
    type SystemData = (
        Write<'a,Input>,
        WriteStorage<'a,Transform<V>>,
        WriteStorage<'a,Camera<V>>,
        WriteStorage<'a,MoveNext<V>>,
        ReadExpect<'a,Player>
    );
    fn run(&mut self, (mut input, mut transforms, mut cameras, mut move_nexts, player) : Self::SystemData) {
        if input.is_camera_movement_enabled() {
            update_camera(&mut input,
                          &mut transforms.get_mut(player.0).unwrap(),
                          &mut cameras.get_mut(player.0).unwrap(),
                          &mut move_nexts.get_mut(player.0).unwrap()
            );
        }
    }
}

fn update_camera<V : VectorTrait>(input : &mut Input, transform: &mut Transform<V>, camera : &mut Camera<V>, move_next : &mut MoveNext<V>)
{
    //clear movement
    *move_next = MoveNext{ next_dpos: None, can_move: Some(true) };
    //limit max dt
    let dt = input.get_dt();

    let mut any_slide_turn = false;

    //mouse
    match input.movement_mode {
        MovementMode::Player(PlayerMovementMode::Mouse) => {
            let (dmx, dmy) = input.mouse.mouse_dpos;
            if dmx.abs() != 0. {
                if input.helper.held_shift() {
                    camera.turn(transform,0,2, dmx*dt*MOUSE_SENSITIVITY);
                } else {
                    camera.turn(transform,0,-1,dmx*dt*MOUSE_SENSITIVITY);
                }
                any_slide_turn = true;
            }
            //y mouse movement
            if dmy.abs() != 0. {

                match (V::DIM, input.helper.held_shift()) {
                    (3, _) | (4, true) => camera.tilt(transform,1,-1,-dmy*dt*MOUSE_SENSITIVITY),
                    (4, false) => camera.turn(transform,2,-1,-dmy*dt*MOUSE_SENSITIVITY),
                    (_, _) => panic!("Invalid dimension"),
                };
                //camera.spin(axis,-1,-my*dt*MOUSE_SENSITIVITY);
                any_slide_turn = true;
            }
        },
        _ => (),
    }

    //keyboard

    //forwards + backwards
    // TODO why do we call update here and not during other operations?
    if input.helper.key_held(MOVE_FORWARDS) {
        move_next.translate(
            camera.get_slide_dpos(camera.heading[-1],dt)
        );
        input.update = true;
    }
    if input.helper.key_held(MOVE_BACKWARDS) {
        move_next.translate(
            camera.get_slide_dpos(-camera.heading[-1],dt)
        );
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
            if input.helper.held_alt() ^ match input.movement_mode {MovementMode::Player(PlayerMovementMode::Mouse) => true, _ => false} {
                move_next.translate(
                    camera.get_slide_dpos(camera.heading[axis]*movement_sign,dt)
                );
                //rotations
            } else {
                //special case : (0,2) rotation
                if V::DIM == 4 && input.helper.held_shift() && axis == 2 {
                    camera.spin(transform,0,2,movement_sign*dt)
                    //turning: rotation along (axis,-1)
                } else {
                    if axis == 1 {
                        camera.tilt(transform,axis,-1,movement_sign*dt);
                    } else {
                        camera.turn(transform,axis,-1,movement_sign*dt);
                    }
                }

            }
        };

    }
    //spin unless turning or sliding
    if V::DIM == 4 && any_slide_turn == false {
        //camera.spin(transform,0,2,0.05*dt);
    }
    //         //reset orientation
    //         if !input.pressed.space {
    //             camera.frame = V::M::id();
    //             camera.update();
    //             input.update = true;
    //             input.pressed.space = true;

}