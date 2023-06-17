use std::f32::consts::PI;
use crate::components::*;
use crate::input::{Input, MOUSE_SENSITIVITY, MovementMode, ShapeMovementMode};
use crate::vector::{VectorTrait,Field,VecIndex};

use glium::glutin;
use glutin::event::VirtualKeyCode as VKC;
use crate::geometry::transform::Scaling;

use super::{ShapeManipulationState, ToggleKeys};
use super::key_map::{MOVE_FORWARDS, MOVE_BACKWARDS, MOVE_KEYMAP, AXIS_KEYMAP, SNAPPING, RESET_ORIENTATION};

const SPEED : Field = 1.5;
const ANG_SPEED : Field = 1.5*PI/3.0;

pub fn get_slide_dpos<V: VectorTrait>(direction : V, time : Field) -> V {
    direction.normalize()*SPEED*time
}

fn mouse_rotation<V: VectorTrait>(input: &Input, dt: Field, transform: &mut Transform<V>) -> bool {
    let mut any_slide_turn = false;
    //mouse
    let (dmx, dmy) = input.mouse.mouse_dpos;
    if dmx.abs() != 0. {
        if input.helper.held_shift() {
            transform.rotate(0, 2, dmx * dt * MOUSE_SENSITIVITY);
        } else {
            transform.rotate(0, -1, dmx * dt * MOUSE_SENSITIVITY);
        }
        any_slide_turn = true;
    }
    //y mouse movement
    if dmy.abs() != 0. {

        match (V::DIM, input.helper.held_shift()) {
            (3, _) | (4, true) => transform.rotate(1,-1,-dmy*dt*MOUSE_SENSITIVITY),
            (4, false) => transform.rotate(2,-1,-dmy*dt*MOUSE_SENSITIVITY),
            (_, _) => panic!("Invalid dimension"),
        };
        //camera.spin(axis,-1,-my*dt*MOUSE_SENSITIVITY);
        any_slide_turn = true;
    }
    return any_slide_turn
}

fn forwards_backwards_movement<V: VectorTrait>(
    input: &Input,
    dt: Field,
    transform: &mut Transform<V>
) -> bool {
    let mut update = false;
    if input.helper.key_held(MOVE_FORWARDS) {
        transform.translate(
            get_slide_dpos(transform.frame[-1],dt)
        );
        update = true;
    }
    if input.helper.key_held(MOVE_BACKWARDS) {
        transform.translate(
            get_slide_dpos(-transform.frame[-1], dt)
        );
        update = true;
    }
    return update
}

fn sliding_and_turning<V: VectorTrait>(
    input: &Input,
    dt: Field,
    transform: &mut Transform<V>
) -> bool {
    let mut any_slide_turn = false;
    for &(key_minus, key_plus, axis) in MOVE_KEYMAP.iter() {

        let movement_sign =
            input.helper.key_held(key_plus) as i32 -
                input.helper.key_held(key_minus) as i32;
        let movement_sign = movement_sign as f32;

        if movement_sign != 0. {
            any_slide_turn = true;
            //sliding
            if input.helper.held_alt() {
                transform.translate(
                    get_slide_dpos(transform.frame[axis]*movement_sign,dt)
                );
                //rotations
            } else {
                //special case : (0,2) rotation
                if V::DIM == 4 && input.helper.held_shift() && axis == 2 {
                    transform.rotate(0,2,movement_sign*dt)
                    //turning: rotation along (axis,-1)
                } else {
                    if axis == 1 {
                        transform.rotate(axis,-1,movement_sign*dt);
                    } else {
                        transform.rotate(axis,-1,movement_sign*dt);
                    }
                }

            }
        };
    }
    return any_slide_turn;
}

fn get_axis<V: VectorTrait>(input: &Input) -> Option<VecIndex> {
    let mut axis = None;
    for (key_code, ax) in AXIS_KEYMAP.iter() {
        if input.helper.key_held(*key_code) & (*ax < V::DIM) {
            axis = Some(*ax)
        }
    }
    axis
}


pub fn set_axes(toggle_keys: &mut ToggleKeys, locked_axes: &mut Vec<VecIndex>, dim: VecIndex) {

    // for (key_code, ax) in AXIS_KEYMAP.iter() {
    //     if input.toggle_keys.state(*key_code) && !locked_axes.contains(ax) {
    //         locked_axes.push(*ax);
    //     }
    //     if !input.toggle_keys.state(*key_code) && locked_axes.contains(ax) {
    //         locked_axes.retain(|x| *x != *ax);
    //     }
    // }
    for (key_code, ax) in AXIS_KEYMAP.iter() {
        if toggle_keys.state(*key_code) && *ax < dim {
            if locked_axes.contains(ax) {
                locked_axes.retain(|x| *x != *ax);
            } else {
                locked_axes.push(*ax);
            }
            toggle_keys.remove(*key_code);
        }
    }
}
fn round_to(x: Field, to: Field) -> Field {
    (x / to).round() * to
}
const ROUND_VEC_RESOLUTION: Field = 0.25;
fn round_vec<V: VectorTrait>(v: V) -> V {
    v.map(|vi| round_to(vi, ROUND_VEC_RESOLUTION))
}

const ROUND_ANGLE_RESOLUTION: Field = PI / 8.0;
fn round_angle(angle: Field) -> Field {
    round_to(angle, ROUND_ANGLE_RESOLUTION)
}

pub fn snapping_enabled(input: &Input) -> bool {
    input.helper.key_held(SNAPPING)
}

pub fn reset_orientation_and_scale<V: VectorTrait>(input: &Input, transform: &mut Transform<V>) {
    if input.helper.key_held(RESET_ORIENTATION) {
        *transform = Transform::new(Some(transform.pos), None, None);
    }
}

pub fn pos_to_grid<V: VectorTrait>(input: &Input, transform: &mut Transform<V>) {
    if input.helper.key_held(SNAPPING) && input.helper.key_held(VKC::LShift) {
        transform.pos = round_vec(transform.pos)
    }
}


pub fn scrolling_axis_translation<V: VectorTrait>(
    input: &Input,
    locked_axes: &Vec<VecIndex>,
    snap: bool,
    original_transform: &Transform<V>,
    pos_delta: V,
    transform: &mut Transform<V>
) -> (bool, V) {
    let mut new_pos_delta = pos_delta;
    let mut update = false;
    let (dx, dy) = input.mouse.mouse_or_scroll_deltas();
    if dx != 0.0 && dy != 0.0 {
        let dpos = match locked_axes.len() {
            0 => V::zero(),
            1 => V::one_hot(locked_axes[0]) * (dx + dy),
            2 => V::one_hot(locked_axes[0]) * dx + V::one_hot(locked_axes[1]) * dy,
            _ => V::zero(),
        } * input.get_dt() * MOUSE_SENSITIVITY;
        new_pos_delta = pos_delta + dpos; 
        *transform = original_transform.clone();
        transform.translate(
            match snap {
                true => round_vec(new_pos_delta),
                false => new_pos_delta
            }
        );
        update = true;
    }
    return (update, new_pos_delta)
}


pub fn axis_rotation<V: VectorTrait>(
    input: &Input,
    locked_axes: &Vec<VecIndex>,
    snap: bool,
    original_transform: &Transform<V>,
    angle_delta: Field,
    transform: &mut Transform<V>
) -> (bool, Field) {
    let mut new_angle_delta = angle_delta;
    let mut update = false;
    let (dx, dy) = input.mouse.mouse_or_scroll_deltas();
    if dx != 0.0 && dy != 0.0 {
        match locked_axes.len() {
            2 => {
                let dangle = (dx + dy) * input.get_dt() * MOUSE_SENSITIVITY;
                new_angle_delta = new_angle_delta + dangle; 
                *transform = original_transform.clone();
                transform.rotate(
                    locked_axes[0],
                    locked_axes[1],
                    match snap {
                        true => round_angle(new_angle_delta),
                        false => new_angle_delta
                    }
                );
                update = true;
            },
            4 => {
                let dangle = (dx + dy) * input.get_dt() * MOUSE_SENSITIVITY;
                new_angle_delta = new_angle_delta + dangle; 
                *transform = original_transform.clone();
                transform.rotate(
                    locked_axes[0],
                    locked_axes[1],
                    match snap {
                        true => round_angle(new_angle_delta),
                        false => new_angle_delta
                    }
                );
                transform.rotate(
                    locked_axes[2],
                    locked_axes[3],
                    match snap {
                        true => round_angle(new_angle_delta),
                        false => new_angle_delta
                    }
                )
                ;
                update = true;
            }
            _ => {}, // 4 would be valid in 4d
        }
    }
    
    return (update, new_angle_delta)
}


pub fn scrolling_axis_scaling<V: VectorTrait>(
    input: &Input,
    locked_axes: &Vec<VecIndex>,
    snap: bool,
    original_transform: &Transform<V>,
    scale_delta: Scaling<V>,
    transform: &mut Transform<V>
) -> (bool, Scaling<V>) {
    let mut new_scale_delta = scale_delta;
    let mut update = false;
    
    let (dx, dy) = input.mouse.mouse_or_scroll_deltas();
    if dx != 0.0 && dy != 0.0 {
        let dscale = match locked_axes.len() {
            1 => V::one_hot(locked_axes[0]) * (dx + dy) * input.get_dt() * MOUSE_SENSITIVITY,
            2 => (V::one_hot(locked_axes[0]) * dx + V::one_hot(locked_axes[1]) * dy) * input.get_dt() * MOUSE_SENSITIVITY,
            _ => V::ones() * (dx + dy) * input.get_dt() * MOUSE_SENSITIVITY,
        };
        let new_scale_delta_vec = match scale_delta {
            Scaling::Scalar(f) =>V::ones() * f + dscale,
            Scaling::Vector(v) => v + dscale,
        }; 
        new_scale_delta = Scaling::Vector(new_scale_delta_vec);
        *transform = original_transform.clone();
        transform.scale(
            Scaling::Vector(
                match snap {
                    true => round_vec(new_scale_delta_vec),
                    false => new_scale_delta_vec
                }
            )
        );
        update = true;
    }
    return (update, new_scale_delta)
}

// TODO rewrite update_camera transformations in terms of these methods; further decompose
// (a bit tricky because of slight differences in rotations)
pub fn update_transform<V : VectorTrait>(
    input : &Input,
    transform: &mut Transform<V>) -> bool
{
    //clear movement
    //*move_next = MoveNext{ next_dpos: None, can_move: Some(true) };
    //limit max dt
    let dt = input.get_dt();

    let mut any_slide_turn = mouse_rotation(input, dt, transform);

    //keyboard

    //forwards + backwards
    let update = forwards_backwards_movement(input, dt, transform);

    //sliding,turning
    any_slide_turn = any_slide_turn | sliding_and_turning(input, dt, transform);

    //         //reset orientation
    //         if !input.pressed.space {
    //             camera.frame = V::M::id();
    //             camera.update();
    //             input.update = true;
    //             input.pressed.space = true;
    return update | any_slide_turn

}