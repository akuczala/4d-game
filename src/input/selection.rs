use super::input_to_transform::{set_axes, snapping_enabled, axis_rotation, reset_orientation_and_scale, pos_to_grid};
use super::key_map::{CANCEL_MANIPULATION, TRANSLATE_MODE, ROTATE_MODE, SCALE_MODE, FREE_MODE, CREATE_SHAPE, DUPLICATE_SHAPE, DELETE_SHAPE};
use super::{Input, MovementMode, MOUSE_SENSITIVITY, ShapeMovementMode, PlayerMovementMode};

use crate::cleanup::DeletedEntities;
use crate::constants::SELECTION_COLOR;
use crate::draw::ShapeTexture;
use crate::draw::draw_line_collection::DrawLineCollection;
use crate::draw::texture::{color_cube, color_cube_texture, fuzzy_color_cube_texture};
use crate::draw::visual_aids::{calc_wireframe_lines, draw_axes};
use crate::ecs_utils::{Componentable, ModSystem};
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

use crate::vector::{VectorTrait,Field,VecIndex, MatrixTrait, barycenter};
use crate::{components::*, camera};

use glutin::event::{Event,WindowEvent};
use crate::geometry::shape::{RefShapes, self};
use crate::input::input_to_transform::{scrolling_axis_scaling, scrolling_axis_translation, update_transform};
use crate::input::ShapeMovementMode::Scale;

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
            original_transform: Transform::identity()
        }
        
    }
}

#[derive(Clone)]
pub enum ShapeManipulationMode<V, M> {
    Translate(V),
    Rotate(Field),
    Scale(Scaling<V>),
    Free(Transform<V, M>)
}
impl<V: VectorTrait> Default for ShapeManipulationMode<V, V::M> {
    fn default() -> Self {
        Self::Translate(V::zero())
    }
}


// todo: adding an "update" flag for shapes will reduce number of updates needed, and decouple some of this stuff
// e.g. update transform -> update shape -> update shape clip state
pub struct ManipulateSelectedShapeSystem<V>(pub PhantomData<V>);
impl <'a, V> System<'a> for ManipulateSelectedShapeSystem<V>
where
        V: VectorTrait + Componentable,
        V::M: Componentable + Clone
{
    type SystemData = (
        Write<'a,Input>, // need write only for snapping
        Write<'a, ShapeManipulationState<V, V::M>>,
        ReadExpect<'a,Player>,
        WriteStorage<'a, Transform<V, V::M>>,
        ReadStorage<'a, MaybeSelected>,
    );
    fn run(&mut self, (
        mut input,
        mut manip_state,
        player,
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

pub fn set_manipulation_mode<V: VectorTrait>(input: &mut Input, manip_state: &mut ShapeManipulationState<V, V::M>, shape_transform: &Transform<V, V::M>) {
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

pub fn cancel_manipulation<V: VectorTrait>(input: &mut Input, manip_state: &ShapeManipulationState<V, V::M>, shape_transform: &mut Transform<V, V::M>) {
    if let MovementMode::Shape(_) = input.movement_mode {
        if input.helper.key_held(CANCEL_MANIPULATION) {
            *shape_transform = manip_state.original_transform;
            input.movement_mode = MovementMode::Player(PlayerMovementMode::Mouse);
        }
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
                transform,
                camera_transform
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
    return update
}
pub struct SelectTargetSystem<V>(pub PhantomData<V>);
impl <'a,V: VectorTrait + Componentable> System<'a> for SelectTargetSystem<V>
{
    type SystemData = (
        Read<'a,Input>,
        ReadExpect<'a,Player>,
        ReadStorage<'a,MaybeTarget<V>>,
        WriteStorage<'a,MaybeSelected>,
        WriteStorage<'a, DrawLineCollection<V>>
    );
    fn run(
        &mut self,
        (
            input,
            player,
            maybe_target_storage,
            mut maybe_selected_storage,
            mut write_draw_line_collection
        ) : Self::SystemData) { 
        if let (true, &MovementMode::Player(_)) = (input.helper.mouse_held(0), &input.movement_mode) {
            let maybe_target = maybe_target_storage.get(player.0).expect("Player has no target component");
            let selected = maybe_selected_storage.get_mut(player.0).expect("Player has no selection component");
            if let Some(Selected { entity }) = selected.0 {
                write_draw_line_collection.remove(entity);
            }
            selected.0 = maybe_target.0.as_ref().map(|target| Selected::new(target.entity))
        }
    }
}

pub struct CreateShapeSystem<V>(pub PhantomData<V>);
impl <'a,V> System<'a> for CreateShapeSystem<V>
where
	V: VectorTrait + Componentable,
	V::SubV: Componentable,
	V::M: Componentable
{
    type SystemData = (
        WriteExpect<'a, Input>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, RefShapes<V>>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Transform<V, V::M>>,
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
            ShapeEntityBuilder::new_convex_from_ref_shape(
                &ref_shapes,
                shape_label,
            )
            .with_transform(Transform::pos(shape_pos))
            .with_scale(Scaling::Scalar(1.0))
            .with_texturing_fn(fuzzy_color_cube_texture)
            .insert(e, &lazy);
            lazy.insert(e, StaticCollider);
            // TODO: add to spatial hash set (use BBox hash system)
            
        }
    }
}

pub struct DuplicateShapeSystem<V>(pub PhantomData<V>);
impl <'a, V, U, M> System<'a> for DuplicateShapeSystem<V>
where
        V: VectorTrait<M=M, SubV = U> + Componentable,
        V::M: Componentable + Clone,
        U: Componentable + VectorTrait
{
    type SystemData = (
        WriteExpect<'a, Input>,
        ReadExpect<'a, Player>,
        ReadStorage<'a, MaybeSelected>,
        ReadExpect<'a, RefShapes<V>>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Transform<V, V::M>>,
        ReadStorage<'a, ShapeLabel>,
        ReadStorage<'a, ShapeTexture<U>>,
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
            shape_textures,
            entities
        ): Self::SystemData) {
        
        if let (true, &Some(Selected{entity: selected_entity, ..})) = (input.toggle_keys.state(DUPLICATE_SHAPE), &maybe_selected_storage.get(player.0).unwrap().0) {
            println!("shape duplicated");
            input.toggle_keys.remove(DUPLICATE_SHAPE);
            let e = entities.create();
            let shape_label = shape_label_storage.get(selected_entity).unwrap().clone();
            let transform: Transform<V, M> = read_transform.get(selected_entity).unwrap().clone();
            ShapeEntityBuilder::new_convex_from_ref_shape(&ref_shapes, shape_label)
                .with_transform(transform)
                .with_texture(shape_textures.get(selected_entity).unwrap().clone())
                .insert(e, &lazy);
            lazy.insert(e, StaticCollider);
            // TODO: add to spatial hash set (use BBox hash system)
            // TODO: copy all shape components to new entity?
        }
    }
}

pub struct DeleteShapeSystem<V>(pub PhantomData<V>);

impl <'a, V> System<'a> for DeleteShapeSystem<V>
where V: Componentable
{
    type SystemData = (
        ReadExpect<'a, Player>,
        WriteStorage<'a, MaybeSelected>,
        WriteExpect<'a, Input>,
        Write<'a, DeletedEntities>,
        Entities<'a>
    );

    fn run(
        &mut self,
        (
            player,
            mut write_maybe_selected,
            mut input,
            mut deleted_entities,
            entities
        ): Self::SystemData
    ) {
        if input.toggle_keys.state(DELETE_SHAPE) {
            println!("Delete shape");
            let mut maybe_selected = write_maybe_selected.get_mut(player.0).unwrap();
            if let Some(selected) = &maybe_selected.0 {
                let e = selected.entity;
                deleted_entities.add(e);
                entities.delete(e).unwrap();
                maybe_selected.0 = None;
            }
            input.toggle_keys.remove(DELETE_SHAPE);
        }
    }
}
pub struct UpdateSelectionBox<V>(pub ModSystem<V>);

impl<'a, V> System<'a> for UpdateSelectionBox<V>
where V: Componentable + VectorTrait
{
    type SystemData = (
        ReadExpect<'a, Player>,
        ReadStorage<'a, Shape<V>>,
        WriteStorage<'a, MaybeSelected>,
        WriteStorage<'a, DrawLineCollection<V>>,
    );

    fn run(
        &mut self,
        (
            player,
            read_shapes,
            mut write_maybe_selected,
            mut write_draw_line_collection,
        ): Self::SystemData
    ) {
        if let Some(MaybeSelected(Some(selected))) = write_maybe_selected.get_mut(player.0) {
            for event in read_shapes.channel().read(self.0.reader_id.as_mut().unwrap()) {
                match event {
                    ComponentEvent::Modified(id) => if *id == selected.entity.id() {
                        write_draw_line_collection.insert(
                            selected.entity,
                            selection_box(read_shapes.get(selected.entity).unwrap())
                        ).expect("Couldn't add selection box!");
                    },
                    _ => {}
                }
            }
        }
    }
    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.0.reader_id = Some(
            WriteStorage::<Shape<V>>::fetch(&world).register_reader()
        );
    }

}

pub fn selection_box<V: VectorTrait>(shape: &Shape<V>) -> DrawLineCollection<V> {
    DrawLineCollection::from_lines(
        calc_wireframe_lines(shape),
        SELECTION_COLOR
    ).extend(
        draw_axes(barycenter(&shape.verts), 1.0)
    )
}