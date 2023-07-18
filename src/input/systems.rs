use glium::backend::glutin::headless;
use specs::prelude::*;
use std::marker::PhantomData;

use crate::cleanup::DeletedEntities;
use crate::config::Config;
use crate::ecs_utils::ModSystem;
use crate::{components::*, config};
use crate::{ecs_utils::Componentable, vector::VectorTrait};

use super::input_to_transform::{pos_to_grid, reset_orientation_and_scale};
use super::{
    cancel_manipulation, create_shape, delete_shape, duplicate_shape, manipulate_shape,
    manipulate_shape_outer, print_debug, selection_box, set_manipulation_mode,
    update_camera::update_camera, Input, MovementMode, ShapeManipulationState,
};

pub struct UpdateCameraSystem<V>(pub PhantomData<V>);
impl<'a, V> System<'a> for UpdateCameraSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        Write<'a, Input>,
        WriteStorage<'a, Transform<V, V::M>>,
        WriteStorage<'a, Camera<V>>,
        WriteStorage<'a, Heading<V::M>>,
        WriteStorage<'a, MoveNext<V>>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, Config>,
    );
    fn run(
        &mut self,
        (mut input, mut transforms, mut cameras, mut headings, mut move_nexts, player, config): Self::SystemData,
    ) {
        if input.is_camera_movement_enabled() {
            update_camera(
                &mut input,
                &config.view,
                transforms.get_mut(player.0).unwrap(),
                headings.get_mut(player.0).unwrap(),
                cameras.get_mut(player.0).unwrap(),
                move_nexts.get_mut(player.0).unwrap(),
            );
        }
    }
}

pub struct ManipulateSelectedShapeSystem<V>(pub PhantomData<V>);
impl<'a, V> System<'a> for ManipulateSelectedShapeSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable + Clone,
{
    type SystemData = (
        Write<'a, Input>, // need write only for snapping
        Write<'a, ShapeManipulationState<V, V::M>>,
        ReadExpect<'a, Player>,
        WriteStorage<'a, Transform<V, V::M>>,
        ReadStorage<'a, MaybeSelected>,
    );
    fn run(
        &mut self,
        (
        mut input,
        mut manip_state,
        player,
        mut transform_storage,
        maybe_selected_storage
    ) : Self::SystemData,
    ) {
        let maybe_selected = maybe_selected_storage.get(player.0).unwrap();
        if let MaybeSelected(Some(Selected { entity, .. })) = maybe_selected {
            // TODO: It's annoying that I have to clone the camera's transform when we know that it is distinct from selected_transform.
            // how to convince rust of this?
            let camera_transform = *transform_storage.get(player.0).unwrap();
            // TODO: this get will always trigger a mutation event
            // we might able to circumvent this by returning an Option<Transfrom> from manipulate_shape_outer
            let selected_transform = transform_storage
                .get_mut(*entity)
                .expect("Selected entity has no Transform");
            manipulate_shape_outer(
                &mut input,
                &mut manip_state,
                selected_transform,
                &camera_transform,
            )
        }
    }
}

pub struct SelectTargetSystem<V>(pub PhantomData<V>);
impl<'a, V: VectorTrait + Componentable> System<'a> for SelectTargetSystem<V> {
    type SystemData = (
        Read<'a, Input>,
        ReadExpect<'a, Config>,
        ReadExpect<'a, Player>,
        ReadStorage<'a, MaybeTarget<V>>,
        WriteStorage<'a, MaybeSelected>,
        WriteStorage<'a, DrawLineCollection<V>>,
    );
    fn run(
        &mut self,
        (
            input,
            config,
            player,
            maybe_target_storage,
            mut maybe_selected_storage,
            mut write_draw_line_collection,
        ): Self::SystemData,
    ) {
        if let (true, &MovementMode::Player(_)) = (
            input.helper.mouse_held(0) & config.editor.enabled,
            &input.movement_mode,
        ) {
            let maybe_target = maybe_target_storage
                .get(player.0)
                .expect("Player has no target component");
            let selected = maybe_selected_storage
                .get_mut(player.0)
                .expect("Player has no selection component");
            if let Some(Selected { entity }) = selected.0 {
                write_draw_line_collection.remove(entity);
            }
            selected.0 = maybe_target
                .0
                .as_ref()
                .map(|target| Selected::new(target.entity))
        }
    }
}

pub struct CreateShapeSystem<V>(pub PhantomData<V>);
impl<'a, V> System<'a> for CreateShapeSystem<V>
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    type SystemData = (
        WriteExpect<'a, Input>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, RefShapes<V>>,
        ReadExpect<'a, Config>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Transform<V, V::M>>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut input, player, ref_shapes, config, lazy, read_transform, entities): Self::SystemData,
    ) {
        if let Some(builder) = create_shape(
            &mut input,
            &ref_shapes,
            &config,
            read_transform.get(player.0).unwrap(),
        ) {
            builder.insert(entities.create(), &lazy);
        }
    }
}

pub struct DuplicateShapeSystem<V>(pub PhantomData<V>);
impl<'a, V, U, M> System<'a> for DuplicateShapeSystem<V>
where
    V: VectorTrait<M = M, SubV = U> + Componentable,
    V::M: Componentable + Clone,
    U: Componentable + VectorTrait,
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
        ReadStorage<'a, StaticCollider>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (
            mut input,
            player,
            maybe_selected_storage,
            ref_shapes,
            lazy,
            read_transform,
            shape_label_storage,
            shape_textures,
            static_colliders,
            entities,
        ): Self::SystemData,
    ) {
        if let &MaybeSelected(Some(Selected {
            entity: selected_entity,
            ..
        })) = maybe_selected_storage.get(player.0).unwrap()
        {
            if let Some(builder) = duplicate_shape(
                &mut input,
                &ref_shapes,
                shape_label_storage.get(selected_entity).unwrap(),
                read_transform.get(selected_entity).unwrap(),
                shape_textures.get(selected_entity).unwrap(),
                static_colliders.get(selected_entity),
            ) {
                builder.insert(entities.create(), &lazy);
            }
        }
    }
}

pub struct DeleteShapeSystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for DeleteShapeSystem<V>
where
    V: Componentable,
{
    type SystemData = (
        ReadExpect<'a, Player>,
        WriteStorage<'a, MaybeSelected>,
        WriteExpect<'a, Input>,
        Write<'a, DeletedEntities>,
    );

    fn run(
        &mut self,
        (player, mut write_maybe_selected, mut input, mut deleted_entities): Self::SystemData,
    ) {
        delete_shape(
            &mut input,
            write_maybe_selected.get_mut(player.0).unwrap(),
            &mut deleted_entities,
        );
    }
}
pub struct UpdateSelectionBox<V>(pub ModSystem<V>);

impl<'a, V> System<'a> for UpdateSelectionBox<V>
where
    V: Componentable + VectorTrait,
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
        ): Self::SystemData,
    ) {
        if let Some(MaybeSelected(Some(selected))) = write_maybe_selected.get_mut(player.0) {
            self.0.for_each_modified(read_shapes.channel(), |id| {
                if *id == selected.entity.id() {
                    write_draw_line_collection
                        .insert(
                            selected.entity,
                            selection_box(read_shapes.get(selected.entity).unwrap()),
                        )
                        .expect("Couldn't add selection box!");
                }
            })
            // for event in read_shapes.channel().read(self.0.reader_id.as_mut().unwrap()) {
            //     match event {
            //         ComponentEvent::Modified(id) =>
            //         _ => {}
            //     }
            // }
        }
    }
    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.0.reader_id = Some(WriteStorage::<Shape<V>>::fetch(world).register_reader());
    }
}

pub struct PrintDebugSystem<V>(pub PhantomData<V>);
impl<'a, V: VectorTrait + Componentable> System<'a> for PrintDebugSystem<V> {
    type SystemData = (Write<'a, Input>, Write<'a, ClipState<V>>);

    fn run(&mut self, (mut input, mut clip_state): Self::SystemData) {
        print_debug::<V>(&mut input, &mut clip_state);
    }
}
