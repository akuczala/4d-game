use super::input_to_transform::{set_axes, snapping_enabled, axis_rotation, reset_orientation_and_scale, pos_to_grid};
use super::key_map::{CANCEL_MANIPULATION, TRANSLATE_MODE, ROTATE_MODE, SCALE_MODE, FREE_MODE, CREATE_SHAPE, DUPLICATE_SHAPE};
use super::{Input, MovementMode, MOUSE_SENSITIVITY, ShapeMovementMode, PlayerMovementMode};

use crate::geometry::transform::{Scaling, self};
use crate::player::Player;
use crate::shape_entity_builder::ShapeEntityBuilder;
use crate::spatial_hash::{SpatialHash, SpatialHashSet};
use std::collections::HashMap;
use std::marker::PhantomData;

use glium::glutin;
use glutin::event::VirtualKeyCode as VKC;
use glutin::event::{TouchPhase,MouseScrollDelta};
use glutin::dpi::LogicalPosition;

use winit_input_helper::WinitInputHelper;

use specs::prelude::*;

use crate::vector::{VectorTrait,Field,VecIndex};
use crate::{components::*, camera};

use glutin::event::{Event,WindowEvent};
use crate::geometry::shape::{RefShapes, self};
use crate::input::input_to_transform::{scrolling_axis_scaling, scrolling_axis_translation, update_transform};
use crate::input::ShapeMovementMode::Scale;

// would have liked to make this part of the Input struct, but I don't feel like adding <V> to every input object.
// Plus it is nice to keep Input dimension agnostic
pub struct ShapeManipulationState<V: VectorTrait> {
    pub locked_axes: Vec<VecIndex>,
    pub mode: ShapeManipulationMode<V>,
    pub snap: bool,
    pub original_transform: Transform<V>,
}
impl<V: VectorTrait> Default for ShapeManipulationState<V> {
    fn default() -> Self {
        Self {
            locked_axes: Vec::new(),
            mode: Default::default(),
            snap: false,
            original_transform: Transform::identity()
        }
        
    }
}

#[derive(Clone)]
pub enum ShapeManipulationMode<V: VectorTrait> {
    Translate(V),
    Rotate(Field),
    Scale(Scaling<V>),
    Free(Transform<V>)
}
impl<V: VectorTrait> Default for ShapeManipulationMode<V> {
    fn default() -> Self {
        Self::Translate(V::zero())
    }
}

// product typy alternative to above
// pub struct ShapeManipulationDelta<V: VectorTrait> {
//     pos: V,
//     angle: Field,
//     scale: Scaling<V>,
//     free: Transform<V>
// }
// impl<V: VectorTrait> Default for ShapeManipulationDelta<V> {
//     fn default() -> Self {
//         Self {
//             pos: V::zero(),
//             angle: 0.0,
//             scale: Scaling::unit(),
//             free: Transform::identity()
//         }
//     }
// }


// todo: adding an "update" flag for shapes will reduce number of updates needed, and decouple some of this stuff
// e.g. update transform -> update shape -> update shape clip state
pub struct ManipulateSelectedShapeSystem<V: VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for ManipulateSelectedShapeSystem<V> {
    type SystemData = (
        Write<'a,Input>, // need write only for snapping
        Write<'a, ShapeManipulationState<V>>,
        ReadExpect<'a,Player>,
        ReadStorage<'a, Camera<V>>,
        WriteStorage<'a,Transform<V>>,
        ReadStorage<'a,MaybeSelected<V>>,
    );
    fn run(&mut self, (
        mut input,
        mut manip_state,
        player,
        camera,
        mut transform_storage,
        maybe_selected_storage
    ) : Self::SystemData) {
        let maybe_selected= maybe_selected_storage.get(player.0).unwrap();
        if let MaybeSelected(Some(Selected{entity,..})) = maybe_selected {
            // TODO: It's annoying that I have to clone the camera's transform when we know that it is distinct from selected_transform.
            // how to convince rust of this?
            let camera_transform = transform_storage.get(player.0).unwrap().clone(); 
            let selected_transform = transform_storage.get_mut(*entity).expect("Selected entity has no Transform");
            set_manipulation_mode(&mut input, &mut manip_state, selected_transform);
            cancel_manipulation(&mut input, &mut manip_state, selected_transform);
            reset_orientation_and_scale(&input, selected_transform);
            pos_to_grid(&input, selected_transform);
            match (&mut input).movement_mode {
                MovementMode::Shape(_) => {
                    manipulate_shape(
                        &mut input,
                        &mut manip_state,
                        selected_transform,
                        &camera_transform,
                    );
                },
                _ => ()
            }
        }
    }
}
pub const MODE_KEYMAP: [(VKC, ShapeMovementMode); 4] = [
    (TRANSLATE_MODE, ShapeMovementMode::Translate),
    (ROTATE_MODE, ShapeMovementMode::Rotate),
    (SCALE_MODE, ShapeMovementMode::Scale),
    (FREE_MODE, ShapeMovementMode::Free)
];

pub fn set_manipulation_mode<V: VectorTrait>(input: &mut Input, manip_state: &mut ShapeManipulationState<V>, shape_transform: &Transform<V>) {
    for &(key, mode) in MODE_KEYMAP.iter() {
        // use key_held here instead of released or pressed because the latter don't seem to work outside of Input.listen_inputs
        if input.helper.key_held(key) {
            input.movement_mode = MovementMode::Shape(mode);
            manip_state.mode = match mode {
                ShapeMovementMode::Translate => ShapeManipulationMode::Translate(V::zero()),
                ShapeMovementMode::Rotate => ShapeManipulationMode::Rotate(0.0),
                ShapeMovementMode::Scale => ShapeManipulationMode::Scale(Scaling::unit()),
                ShapeMovementMode::Free => ShapeManipulationMode::Free(Transform::identity())
            };
            manip_state.original_transform = shape_transform.clone();
            manip_state.locked_axes = Vec::new();
        }
        if input.helper.mouse_held(0) {
            // back to player movement mode?
            // will this cause annoying accidental selections?
        }
    }
}

pub fn cancel_manipulation<V: VectorTrait>(input: &mut Input, manip_state: &ShapeManipulationState<V>, shape_transform: &mut Transform<V>) {
    if let MovementMode::Shape(_) = input.movement_mode {
        if input.helper.key_held(CANCEL_MANIPULATION) {
            *shape_transform = manip_state.original_transform;
            input.movement_mode = MovementMode::Player(PlayerMovementMode::Mouse);
        }
    }
}

pub fn manipulate_shape<V: VectorTrait>(
    input: &mut Input,
    manip_state: &mut ShapeManipulationState<V>,
    transform: &mut Transform<V>,
    camera_transform: &Transform<V>,
) {
    // TODO: align movement with camera frame
    set_axes(&mut input.toggle_keys, &mut manip_state.locked_axes, V::DIM);
    manip_state.snap = snapping_enabled(input);
    //let new_mode;
    let (update, new_mode) = match manip_state.mode {
        ShapeManipulationMode::Translate(pos_delta) => {

            let (u, d) = scrolling_axis_translation(
                input, 
                &manip_state.locked_axes,
                manip_state.snap,
                &manip_state.original_transform, 
                pos_delta,
                transform,
                camera_transform,
            );
            (u, ShapeManipulationMode::Translate(d))
        },
        ShapeManipulationMode::Rotate(angle_delta) => {
            let (u, new_angle_delta) = axis_rotation(
                input,
                &manip_state.locked_axes,
                manip_state.snap,
                &manip_state.original_transform,
                angle_delta,
                transform
            );
            (u, ShapeManipulationMode::Rotate(new_angle_delta))
        },
        ShapeManipulationMode::Scale(scale_delta) => {
            let (u, new_scale_delta) = scrolling_axis_scaling(
                input,
                &manip_state.locked_axes,
                manip_state.snap,
                &manip_state.original_transform,
                scale_delta,
                transform
            );
            (u, ShapeManipulationMode::Scale(new_scale_delta))
        },
        //this mode allows you to control the shape as if it were the camera
        ShapeManipulationMode::Free(transform_delta) => {
            let mut new_transform_delta = transform_delta.clone();
            let update = update_transform(input, &mut new_transform_delta);
            *transform = manip_state.original_transform.clone().with_transform(new_transform_delta);
            (update, ShapeManipulationMode::Free(new_transform_delta))
        }
    };
    manip_state.mode = new_mode;
    if update {
        
    }
}
pub struct SelectTargetSystem<V: VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for SelectTargetSystem<V> {
    type SystemData = (
        Read<'a,Input>,
        ReadExpect<'a,Player>,
        ReadStorage<'a,Shape<V>>,
        ReadStorage<'a,MaybeTarget<V>>,
        WriteStorage<'a,MaybeSelected<V>>
    );
    fn run(&mut self, (input, player, shape_storage, maybe_target_storage, mut maybe_selected_storage) : Self::SystemData) { 
        if let (true, &MovementMode::Player(_)) = (input.helper.mouse_held(0), &input.movement_mode) {
            let maybe_target = maybe_target_storage.get(player.0).expect("Player has no target component");
            let selected = maybe_selected_storage.get_mut(player.0).expect("Player has no selection component");
            if let MaybeTarget(Some(target)) = maybe_target  {
                // let selected_bbox =  bbox_storage.get(target.entity).expect("Target entity has no bbox");
                // selected.0 = Some(Selected::new_from_bbox(target.entity, selected_bbox));
                let selected_shape =  shape_storage.get(target.entity).expect("Target entity has no shape");
                selected.0 = Some(Selected::new_from_shape(target.entity, selected_shape));
            } else {
                selected.0 = None
            }
        }
    }
}

pub struct CreateShapeSystem<V: VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for CreateShapeSystem<V> {
    type SystemData = (
        WriteExpect<'a, Input>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, RefShapes<V>>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Transform<V>>,
        Entities<'a>,
    );

    fn run(
        &mut self, (
            mut input,
            player ,
            ref_shapes,
            lazy,
            read_transform,
            entities
        ): Self::SystemData) {
        
        if input.toggle_keys.state(CREATE_SHAPE) {
            println!("shape created");
            input.toggle_keys.remove(CREATE_SHAPE);
            let player_transform = read_transform.get(player.0).unwrap();
            let pos = player_transform.pos;
            let dir = player_transform.frame[-1];
            let shape_pos = pos + dir * 2.0;
            let e = entities.create();
            let shape_label = ShapeLabel("Cube".to_string());
            ShapeEntityBuilder::convex_from_ref_shape(
                &ref_shapes,
                shape_label,
            )
            .with_transform(Transform::pos(shape_pos))
            .with_scale(Scaling::Scalar(0.5))
            .insert(e, &lazy);
            lazy.insert(e, StaticCollider);
            // TODO: add to spatial hash set (use BBox hash system)
            
        }
    }
}

pub struct DuplicateShapeSystem<V: VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for DuplicateShapeSystem<V> {
    type SystemData = (
        WriteExpect<'a, Input>,
        ReadExpect<'a, Player>,
        ReadStorage<'a, MaybeSelected<V>>,
        ReadExpect<'a, RefShapes<V>>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Transform<V>>,
        ReadStorage<'a, ShapeLabel>,
        Entities<'a>,
    );

    fn run(
        &mut self, (
            mut input,
            player ,
            maybe_selected_storage,
            ref_shapes,
            lazy,
            read_transform,
            shape_label_storage,
            entities
        ): Self::SystemData) {
        
        if let (true, &Some(Selected{entity: selected_entity, ..})) = (input.toggle_keys.state(DUPLICATE_SHAPE), &maybe_selected_storage.get(player.0).unwrap().0) {
            println!("shape duplicated");
            input.toggle_keys.remove(DUPLICATE_SHAPE);
            let e = entities.create();
            let shape_label = shape_label_storage.get(selected_entity).unwrap().clone();
            ShapeEntityBuilder::convex_from_ref_shape(&ref_shapes, shape_label)
                .with_transform(read_transform.get(selected_entity).unwrap().clone())
                .insert(e, &lazy);
            lazy.insert(e, StaticCollider);
            // TODO: add to spatial hash set (use BBox hash system)
            
        }
    }
}