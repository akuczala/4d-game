use std::f32::consts::PI;
use crate::components::*;
use crate::input::{Input, MOUSE_SENSITIVITY, MovementMode, ShapeMovementMode};
use crate::vector::{VectorTrait,Field,VecIndex};

use glium::glutin;
use glutin::event::VirtualKeyCode as VKC;
use crate::geometry::transform::Scaling;

use super::ShapeManipulationState;
use super::key_map::{MOVE_FORWARDS, MOVE_BACKWARDS, MOVE_KEYMAP, AXIS_KEYMAP, SNAPPING};

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

// TODO: Would be nicer to have axes toggle on + off with same key
pub fn set_axes(input: &Input, locked_axes: &mut Vec<VecIndex>, dim: VecIndex) {
    if input.helper.key_held(VKC::Key0) {
        *locked_axes = Vec::new();
    }
    for (key_code, ax) in AXIS_KEYMAP.iter() {
        if input.helper.key_held(*key_code) & (*ax < dim) & !locked_axes.contains(ax) {
            locked_axes.push(*ax);
        }
    }
}
const ROUND_RESOLUTION: Field = 0.25;
fn round_vec<V: VectorTrait>(v: V) -> V {
    v.map(|vi| (vi /ROUND_RESOLUTION).round() * ROUND_RESOLUTION)
}

pub fn snapping_enabled(input: &Input) -> bool {
    input.helper.key_held(SNAPPING)
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
    if let Some((dx, dy)) = input.mouse.scroll_dpos {
        let dpos = match locked_axes.len() {
            0 => V::zero(),
            1 => V::one_hot(locked_axes[0]) * (dx + dy) * input.get_dt() * MOUSE_SENSITIVITY,
            2 => (V::one_hot(locked_axes[0]) * dx + V::one_hot(locked_axes[1]) * dy) * input.get_dt() * MOUSE_SENSITIVITY,
            _ => V::zero(),
        };
        new_pos_delta = pos_delta + dpos; 
            *transform = original_transform.clone();
            transform.translate(
                match snap {
                    true => round_vec(new_pos_delta),
                    false => new_pos_delta
                }
            );
            update = true;
        // if let Some(axis) = get_axis::<V>(input) {
        //     let dpos = 
        //     //if input.mouse.integrated_scroll_dpos.1.abs() > 100.0 {

        //     //let dpos = V::one_hot(axis) * input.mouse.integrated_scroll_dpos.1.signum() * 0.5;
            
        //     //input.mouse.integrated_scroll_dpos = Default::default(); 
            
        // }
    }
    return (update, new_pos_delta)
}

pub fn scrolling_axis_scaling<V: VectorTrait>(input: &Input, transform: &mut Transform<V>) -> bool{
    let mut update = false;
    if let Some((dx,dy)) = input.mouse.scroll_dpos {
        if let Some(axis) = get_axis::<V>(input) {
            let dscale = V::ones() + V::one_hot(axis) * (dx + dy) * input.get_dt() * MOUSE_SENSITIVITY;
            transform.scale(Scaling::Vector(dscale));
            update = true;
        }
    }
    return update
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