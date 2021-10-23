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

pub enum ShapeMovementMode {
    Translate, Rotate, Scale
}

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
        Entities<'a>
    );
    fn run(&mut self, (
        input, player, ref_shapes, mut shape_storage, mut shape_type_storage, label_storage, mut transform_storage,
        mut maybe_selected_storage, entities) : Self::SystemData) {
        let mut maybe_selected= maybe_selected_storage.get_mut(player.0).unwrap();
        if let MaybeSelected(Some(Selected{entity,..})) = maybe_selected {
            let (selected_shape, selected_shape_type, selected_label, selected_transform) =
                (&mut shape_storage, &mut shape_type_storage, &label_storage, &mut transform_storage).join().get(*entity, &entities)
                    .expect("Selected entity either has no Shape, ShapeLabel, or Transform");
            let selected_ref_shape = ref_shapes.get(selected_label).expect("No reference shape with that name");
            manipulate_shape(&input, selected_shape, selected_shape_type, selected_ref_shape, selected_transform);
        }
    }
}
const MODE_KEYMAP: [(VKC, ShapeMovementMode); 3] = [
    (VKC::G,ShapeMovementMode::Translate),
    (VKC::R,ShapeMovementMode::Rotate),
    (VKC::S,ShapeMovementMode::Scale)
];
const AXIS_KEYMAP: [(VKC, VecIndex); 4] = [(VKC::X, 0), (VKC::Y, 1), (VKC::Z, 2), (VKC::W, 3)];

pub fn manipulate_shape<V: VectorTrait>(
    input: &Input, shape: &mut Shape<V>, shape_type: &mut ShapeType<V>, ref_shape: &Shape<V>, transform: &mut Transform<V>) {
    //println!("scroll diff {:?}",input.helper.scroll_diff());
    if let Some((dx,dy)) = input.scroll_dpos {
        let mut axis = None;
        for (key_code, ax) in AXIS_KEYMAP.iter() {
            if input.helper.key_held(*key_code) & (*ax < V::DIM) {
                axis = Some(ax)
            }
        }
        if let Some(axis) = axis {
            let dpos = V::one_hot(*axis) * (dx + dy) * input.get_dt() * MOUSE_SENSITIVITY;
            transform.translate(dpos);
            shape.update_from_ref(ref_shape, transform);
            if let ShapeType::SingleFace(single_face) = shape_type {
                single_face.update(&shape)
            }
        }
    }
}
pub struct SelectTargetSystem<V: VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for SelectTargetSystem<V> {
    type SystemData = (
        Read<'a,Input>,
        ReadExpect<'a,Player>,
        ReadStorage<'a,BBox<V>>,
        ReadStorage<'a,Shape<V>>,
        ReadStorage<'a,MaybeTarget<V>>,
        WriteStorage<'a,MaybeSelected<V>>
    );
    fn run(&mut self, (input, player, bbox_storage, shape_storage, maybe_target_storage, mut maybe_selected_storage) : Self::SystemData) {
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