use super::input_to_transform::{
    axis_rotation, pos_to_grid, reset_orientation_and_scale, set_axes, snapping_enabled,
};
use super::key_map::{
    CANCEL_MANIPULATION, CREATE_SHAPE, DELETE_SHAPE, DUPLICATE_SHAPE, FREE_MODE, ROTATE_MODE,
    SCALE_MODE, TRANSLATE_MODE,
};
use super::{Input, MovementMode, PlayerMovementMode, ShapeMovementMode, MOUSE_SENSITIVITY};

use crate::cleanup::DeletedEntities;
use crate::config::Config;
use crate::constants::{CUBE_LABEL_STR, SELECTION_COLOR};
use crate::draw::draw_line_collection::DrawLineCollection;
use crate::draw::texture::{color_cube, color_cube_texture, fuzzy_color_cube_texture};
use crate::draw::visual_aids::{calc_wireframe_lines, draw_axes};
use crate::draw::ShapeTexture;
use crate::ecs_utils::Componentable;
use crate::geometry::transform::{self, Scaling};
use crate::player::Player;
use crate::shape_entity_builder::{ShapeEntityBuilder, ShapeEntityBuilderV};
use crate::spatial_hash::{SpatialHash, SpatialHashSet};
use std::collections::HashMap;
use std::marker::PhantomData;

use glium::glutin;
use glutin::dpi::LogicalPosition;
use glutin::event::VirtualKeyCode as VKC;
use glutin::event::{MouseScrollDelta, TouchPhase};

use specs::{Entity, WriteStorage};
use winit_input_helper::WinitInputHelper;

use crate::vector::{barycenter, Field, MatrixTrait, VecIndex, VectorTrait};
use crate::{camera, components::*};

use crate::geometry::shape::{self, RefShapes};
use crate::input::input_to_transform::{
    scrolling_axis_scaling, scrolling_axis_translation, update_transform,
};
use crate::input::ShapeMovementMode::Scale;
use glutin::event::{Event, WindowEvent};

// would have liked to make this part of the Input struct, but I don't feel like adding <V> to every input object.
// Plus it is nice to keep Input dimension agnostic
pub struct ShapeManipulationState<V, M> {
    pub locked_axes: Vec<VecIndex>,
    pub mode: ShapeManipulationMode<V, M>,
    pub snap: bool,
    pub original_transform: Transform<V, M>,
}
impl<V: VectorTrait> Default for ShapeManipulationState<V, V::M> {
    fn default() -> Self {
        Self {
            locked_axes: Vec::new(),
            mode: Default::default(),
            snap: false,
            original_transform: Transform::identity(),
        }
    }
}

#[derive(Clone)]
pub enum ShapeManipulationMode<V, M> {
    Translate(V),
    Rotate(Field),
    Scale(Scaling<V>),
    Free(Transform<V, M>),
}
impl<V: VectorTrait> Default for ShapeManipulationMode<V, V::M> {
    fn default() -> Self {
        Self::Translate(V::zero())
    }
}

pub const MODE_KEYMAP: [(VKC, ShapeMovementMode); 4] = [
    (TRANSLATE_MODE, ShapeMovementMode::Translate),
    (ROTATE_MODE, ShapeMovementMode::Rotate),
    (SCALE_MODE, ShapeMovementMode::Scale),
    (FREE_MODE, ShapeMovementMode::Free),
];

pub fn set_manipulation_mode<V: VectorTrait>(
    input: &mut Input,
    manip_state: &mut ShapeManipulationState<V, V::M>,
    shape_transform: &Transform<V, V::M>,
) {
    for &(key, mode) in MODE_KEYMAP.iter() {
        // use key_held here instead of released or pressed because the latter don't seem to work outside of Input.listen_inputs
        if input.helper.key_held(key) {
            input.movement_mode = MovementMode::Shape(mode);
            manip_state.mode = match mode {
                ShapeMovementMode::Translate => ShapeManipulationMode::Translate(V::zero()),
                ShapeMovementMode::Rotate => ShapeManipulationMode::Rotate(0.0),
                ShapeMovementMode::Scale => ShapeManipulationMode::Scale(Scaling::unit()),
                ShapeMovementMode::Free => ShapeManipulationMode::Free(Transform::identity()),
            };
            manip_state.original_transform = *shape_transform;
            manip_state.locked_axes = Vec::new();
        }
        if input.helper.mouse_held(0) {
            // back to player movement mode?
            // will this cause annoying accidental selections?
        }
    }
}

pub fn cancel_manipulation<V: VectorTrait>(
    input: &mut Input,
    manip_state: &ShapeManipulationState<V, V::M>,
    shape_transform: &mut Transform<V, V::M>,
) {
    if let MovementMode::Shape(_) = input.movement_mode {
        if input.helper.key_held(CANCEL_MANIPULATION) {
            *shape_transform = manip_state.original_transform;
            input.movement_mode = MovementMode::Player(PlayerMovementMode::Mouse);
        }
    }
}

pub fn manipulate_shape_outer<V: VectorTrait>(
    input: &mut Input,
    manip_state: &mut ShapeManipulationState<V, V::M>,
    selected_transform: &mut Transform<V, V::M>,
    camera_transform: &Transform<V, V::M>,
) {
    set_manipulation_mode(input, manip_state, selected_transform);
    cancel_manipulation(input, manip_state, selected_transform);
    reset_orientation_and_scale(input, selected_transform);
    pos_to_grid(input, selected_transform);
    if let MovementMode::Shape(_) = input.movement_mode {
        manipulate_shape(input, manip_state, selected_transform, camera_transform);
    }
}

pub fn manipulate_shape<V: VectorTrait>(
    input: &mut Input,
    manip_state: &mut ShapeManipulationState<V, V::M>,
    transform: &mut Transform<V, V::M>,
    camera_transform: &Transform<V, V::M>,
) -> bool {
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
        }
        ShapeManipulationMode::Rotate(angle_delta) => {
            let (u, new_angle_delta) = axis_rotation(
                input,
                &manip_state.locked_axes,
                manip_state.snap,
                &manip_state.original_transform,
                angle_delta,
                transform,
            );
            (u, ShapeManipulationMode::Rotate(new_angle_delta))
        }
        ShapeManipulationMode::Scale(scale_delta) => {
            let (u, new_scale_delta) = scrolling_axis_scaling(
                input,
                &manip_state.locked_axes,
                manip_state.snap,
                &manip_state.original_transform,
                scale_delta,
                transform,
            );
            (u, ShapeManipulationMode::Scale(new_scale_delta))
        }
        //this mode allows you to control the shape as if it were the camera
        ShapeManipulationMode::Free(transform_delta) => {
            let mut new_transform_delta = transform_delta;
            let update = update_transform(input, &mut new_transform_delta);
            *transform = manip_state
                .original_transform
                .with_transform(new_transform_delta);
            (update, ShapeManipulationMode::Free(new_transform_delta))
        }
    };
    manip_state.mode = new_mode;
    update
}

pub fn selection_box<V: VectorTrait>(shape: &Shape<V>) -> DrawLineCollection<V> {
    DrawLineCollection::from_lines(calc_wireframe_lines(shape), SELECTION_COLOR)
        .extend(draw_axes(barycenter(&shape.verts), 1.0))
}

pub fn create_shape<V: VectorTrait>(
    input: &mut Input,
    ref_shapes: &RefShapes<V>,
    config: &Config,
    player_transform: &Transform<V, V::M>,
) -> Option<ShapeEntityBuilderV<V>> {
    input.toggle_keys.trigger_once(CREATE_SHAPE, || {
        println!("shape created");
        //let player_transform =
        let pos = player_transform.pos;
        let dir = player_transform.frame[-1];
        let shape_pos = pos + dir * 2.0;
        ShapeEntityBuilder::new_convex_from_ref_shape(
            ref_shapes,
            ShapeLabel::from_str(CUBE_LABEL_STR),
        )
        .with_transform(Transform::pos(shape_pos))
        .with_scale(Scaling::Scalar(1.0))
        .with_texturing_fn(|shape| fuzzy_color_cube_texture(shape, config.fuzz_lines.face_num))
        .with_collider(Some(StaticCollider))
        // TODO: add to spatial hash set (use BBox hash system)
    })
}

pub fn duplicate_shape<V: VectorTrait>(
    input: &mut Input,
    ref_shapes: &RefShapes<V>,
    shape_label: &ShapeLabel,
    shape_transform: &Transform<V, V::M>,
    shape_texture: &ShapeTexture<V::SubV>,
    shape_collider: Option<&StaticCollider>,
) -> Option<ShapeEntityBuilderV<V>> {
    input.toggle_keys.trigger_once(DUPLICATE_SHAPE, || {
        println!("shape duplicated");
        ShapeEntityBuilder::new_convex_from_ref_shape(ref_shapes, shape_label.clone())
            .with_transform(*shape_transform)
            .with_texture(shape_texture.clone())
            .with_collider(shape_collider.cloned())
        // TODO: add to spatial hash set (use BBox hash system)
        // TODO: copy all shape components to new entity?
    })
}

pub fn delete_shape(
    input: &mut Input,
    maybe_selected: &mut MaybeSelected,
    deleted_entities: &mut DeletedEntities,
) -> Option<Entity> {
    input.toggle_keys.trigger_once_bind(DELETE_SHAPE, || {
        println!("Delete shape");
        if let Some(selected) = &maybe_selected.0 {
            let e = selected.entity;
            deleted_entities.add(e);
            maybe_selected.0 = None;
            Some(e)
        } else {
            None
        }
    })
}

pub fn update_selection_box() {
    todo!()
}
