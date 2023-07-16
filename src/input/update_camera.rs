use super::input_to_transform::get_slide_dpos;
use super::key_map::{MOVE_BACKWARDS, MOVE_FORWARDS, MOVE_KEYMAP};
use super::{Input, MovementMode, MOUSE_SENSITIVITY};

use crate::config::ViewConfig;
use crate::constants::{ANG_SPEED, MAX_TILT, SPEED};
use crate::ecs_utils::Componentable;
use crate::player::Player;
use std::marker::PhantomData;

use glium::glutin;
use glutin::dpi::LogicalPosition;
use glutin::event::VirtualKeyCode as VKC;
use glutin::event::{MouseScrollDelta, TouchPhase};

use winit_input_helper::WinitInputHelper;

use specs::prelude::*;

use crate::components::*;
use crate::vector::{rotation_matrix, Field, MatrixTrait, VecIndex, VectorTrait};

use crate::geometry::shape::RefShapes;
use crate::input::{PlayerMovementMode, ShapeMovementMode};
use glutin::event::{Event, WindowEvent};

//heading-based rotation affecting both frame and heading
pub fn delta_turn_matrix<V: VectorTrait>(
    heading: &V::M,
    axis1: VecIndex,
    axis2: VecIndex,
    speed_mult: Field,
) -> V::M {
    rotation_matrix(heading[axis1], heading[axis2], Some(speed_mult * ANG_SPEED))
}

//heading-based rotation affecting only camera direction
fn delta_tilt_matrix<V: VectorTrait>(
    heading: &V::M,
    transform: &Transform<V, V::M>,
    axis1: VecIndex,
    axis2: VecIndex,
    speed_mult: Field,
) -> Option<V::M> {
    let dot = heading[axis1].dot(transform.frame[axis2]); // get projection of frame axis along heading axis

    (dot * speed_mult < 0. || dot.abs() < MAX_TILT).then(
        //rotate if tilting direction is opposite projection or if < max tilt
        || {
            rotation_matrix(
                transform.frame[axis1],
                transform.frame[axis2],
                Some(speed_mult * ANG_SPEED),
            )
        },
    )
}

fn turn<V: VectorTrait>(
    heading: &mut Heading<V::M>,
    transform: &mut Transform<V, V::M>,
    axes: (VecIndex, VecIndex),
    speed_mult: Field,
) {
    let dmat = delta_turn_matrix::<V>(&heading.0, axes.0, axes.1, speed_mult);
    transform.frame = transform.frame.dot(dmat);
    heading.0 = heading.0.dot(dmat);
}

fn tilt<V: VectorTrait>(
    heading: &Heading<V::M>,
    transform: &mut Transform<V, V::M>,
    axes: (VecIndex, VecIndex),
    speed_mult: Field,
) {
    if let Some(rot) = delta_tilt_matrix(&heading.0, transform, axes.0, axes.1, speed_mult) {
        transform.frame = transform.frame.dot(rot);
    }
}

pub fn update_camera<V: VectorTrait>(
    input: &mut Input,
    view_config: &ViewConfig,
    transform: &mut Transform<V, V::M>,
    heading: &mut Heading<V::M>,
    camera: &mut Camera<V>,
    move_next: &mut MoveNext<V>,
) {
    //clear movement
    *move_next = MoveNext {
        next_dpos: None,
        can_move: Some(true),
    };
    //limit max dt
    let dt = input.get_dt();

    let mut any_slide_turn = false;

    //mouse
    if let MovementMode::Player(PlayerMovementMode::Mouse) = input.movement_mode {
        let (dmx, dmy) = input.mouse.mouse_dpos;
        if dmx.abs() != 0. {
            if input.helper.held_shift() {
                turn(heading, transform, (0, 2), dmx * dt * MOUSE_SENSITIVITY);
            } else {
                turn(heading, transform, (0, -1), dmx * dt * MOUSE_SENSITIVITY);
            }
            any_slide_turn = true;
        }
        //y mouse movement
        if dmy.abs() != 0. {
            match (V::DIM, input.helper.held_shift()) {
                (3, _) | (4, true) => {
                    tilt(heading, transform, (1, -1), -dmy * dt * MOUSE_SENSITIVITY)
                }
                (4, false) => turn(heading, transform, (2, -1), -dmy * dt * MOUSE_SENSITIVITY),
                (_, _) => panic!("Invalid dimension"),
            };
            //camera.spin(axis,-1,-my*dt*MOUSE_SENSITIVITY);
            any_slide_turn = true;
        }
    }

    //keyboard

    //forwards + backwards
    // TODO why do we call update here and not during other operations?
    if input.helper.key_held(MOVE_FORWARDS) {
        move_next.translate(get_slide_dpos(heading.0[-1], SPEED, dt));
        input.update = true;
    }
    if input.helper.key_held(MOVE_BACKWARDS) {
        move_next.translate(get_slide_dpos(-heading.0[-1], SPEED, dt));
        input.update = true;
    }

    //sliding,turning
    for &(key_minus, key_plus, axis) in MOVE_KEYMAP.iter() {
        let movement_sign =
            input.helper.key_held(key_plus) as i32 - input.helper.key_held(key_minus) as i32;
        let movement_sign = movement_sign as f32;

        if movement_sign != 0. {
            any_slide_turn = true;
            //sliding
            if input.helper.held_alt()
                ^ matches!(
                    input.movement_mode,
                    MovementMode::Player(PlayerMovementMode::Mouse)
                )
            {
                move_next.translate(get_slide_dpos(heading.0[axis] * movement_sign, SPEED, dt));
                //rotations
            } else {
                //special case : (0,2) rotation
                if V::DIM == 4 && input.helper.held_shift() && axis == 2 {
                    turn(heading, transform, (0, 2), movement_sign * dt);
                    //turning: rotation along (axis,-1)
                } else if axis == 1 {
                    tilt(heading, transform, (axis, -1), movement_sign * dt);
                } else {
                    turn(heading, transform, (axis, -1), movement_sign * dt);
                }
            }
        };
    }
    //spin unless turning or sliding
    if V::DIM == 4 && !any_slide_turn {
        turn(heading, transform, (0, 2), view_config.spin_speed * dt);
    }
    camera.update(transform);
    //         //reset orientation
    //         if !input.pressed.space {
    //             camera.frame = V::M::id();
    //             camera.update();
    //             input.update = true;
    //             input.pressed.space = true;
}
