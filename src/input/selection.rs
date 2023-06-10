use super::{Input, MovementMode, MOUSE_SENSITIVITY};

use crate::player::Player;
use std::collections::HashMap;
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
use crate::input::input_to_transform::{scrolling_axis_scaling, scrolling_axis_translation, update_transform};
use crate::input::ShapeMovementMode::Scale;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ShapeMovementMode {
    Translate, Rotate, Scale, Free
}

// todo: adding an "update" flag for shapes will reduce number of updates needed, and decouple some of this stuff
// e.g. update transform -> update shape -> update shape clip state
pub struct ManipulateSelectedShapeSystem<V: VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for ManipulateSelectedShapeSystem<V> {
    type SystemData = (
        Read<'a,Input>,
        ReadExpect<'a,Player>,
        ReadExpect<'a,RefShapes<V>>,
        WriteStorage<'a,Shape<V>>,
        WriteStorage<'a,ShapeType<V>>,
        ReadStorage<'a,ShapeLabel>,
        WriteStorage<'a,Transform<V>>,
        WriteStorage<'a,MaybeSelected<V>>,
        WriteStorage<'a, ShapeClipState<V>>,
        Entities<'a>
    );
    fn run(&mut self, (
        input,
        player,
        ref_shapes,
        mut shape_storage,
        mut shape_type_storage,
        label_storage,
        mut transform_storage,
        mut maybe_selected_storage,
        mut shape_clip_state_storage,
        entities
    ) : Self::SystemData) {
        let maybe_selected= maybe_selected_storage.get_mut(player.0).unwrap();
        if let MaybeSelected(Some(Selected{entity,..})) = maybe_selected {
            let (
                selected_shape,
                selected_shape_type,
                selected_label,
                selected_transform,
                selected_shape_clip_state
            ) =
                (
                    &mut shape_storage,
                    &mut shape_type_storage,
                    &label_storage,
                    &mut transform_storage,
                    &mut shape_clip_state_storage,
                ).join()
                .get(*entity, &entities)
                .expect("Selected entity either has no Shape, ShapeLabel, Transform, or clip state");
            let selected_ref_shape = ref_shapes.get(selected_label).expect("No reference shape with that name");
            match (&input).movement_mode {
                MovementMode::Shape(mode) => manipulate_shape(
                    &input,
                    mode,
                    selected_shape,
                    selected_shape_type,
                    selected_ref_shape,
                    selected_transform,
                    selected_shape_clip_state
                ),
                _ => ()
            }
        }
    }
}
pub const MODE_KEYMAP: [(VKC, ShapeMovementMode); 4] = [
    (VKC::T,ShapeMovementMode::Translate),
    (VKC::R,ShapeMovementMode::Rotate),
    (VKC::Y,ShapeMovementMode::Scale),
    (VKC::F, ShapeMovementMode::Free)
];

// TODO: manipulated shapes do not clip properly - do we need to move it in the spatial hash?
// todo: update spatial hash of updated shapes
pub fn manipulate_shape<V: VectorTrait>(
    input: &Input,
    shape_movement_mode: ShapeMovementMode,
    shape: &mut Shape<V>,
    shape_type: &mut ShapeType<V>,
    ref_shape: &Shape<V>,
    transform: &mut Transform<V>,
    selected_shape_clip_state: &mut ShapeClipState<V>
) {
    //println!("scroll diff {:?}",input.helper.scroll_diff());
    let mut update = false;
    match shape_movement_mode {
        ShapeMovementMode::Translate => {
            update = update | scrolling_axis_translation(input, transform);
        },
        // this mode allows you to control the shape as if it were the camera
        ShapeMovementMode::Free => {
            update = update_transform(input, transform);
        },
        ShapeMovementMode::Scale => {
            update = scrolling_axis_scaling(input, transform)
        }
        _ => {}
    }
    if update {
        shape.update_from_ref(ref_shape, &transform);
        if let ShapeType::SingleFace(single_face) = shape_type {
            single_face.update(&shape)
        }
        // clear static separators for shape, which will be repopulated next draw
        // this is not enough - likely need to also clear separator key for this entity for all other shapes
        selected_shape_clip_state.separators = HashMap::new();
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
        if input.helper.mouse_held(0) {
            let maybe_target = maybe_target_storage.get(player.0).expect("Player has no target component");
            if let MaybeTarget(Some(target)) = maybe_target  {
                let selected = maybe_selected_storage.get_mut(player.0).expect("Player has no selection component");
                // let selected_bbox =  bbox_storage.get(target.entity).expect("Target entity has no bbox");
                // selected.0 = Some(Selected::new_from_bbox(target.entity, selected_bbox));
                let selected_shape =  shape_storage.get(target.entity).expect("Target entity has no shape");
                selected.0 = Some(Selected::new_from_shape(target.entity, selected_shape));
            }
        }
    }
}