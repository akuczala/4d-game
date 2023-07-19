use std::collections::{HashMap, HashSet};

use specs::prelude::*;
use specs::{ReadStorage, System, World, WriteStorage};

use crate::components::{
    BBox, HasBBox, MaybeSelected, Player, ShapeClipState, ShapeLabel, ShapeType,
};
use crate::ecs_utils::Componentable;
use crate::geometry::shape::RefShapes;
use crate::vector::MatrixTrait;
use crate::{
    components::{BBall, Shape, Transform},
    ecs_utils::ModSystem,
    vector::VectorTrait,
};

//TODO: we don't always need to update all of this when a shape gets mutated - e.g. spinning coins do not change any of these components.
// maybe we can include a marker or something to indicate that these shapes should be excluded?
// or something more sophisticated where we indicate when pos and/or rotation have been updated?
#[derive(Default)]
pub struct UpdateBBallSystem<V>(pub ModSystem<V>);

impl<'a, V> System<'a> for UpdateBBallSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadStorage<'a, Shape<V>>,
        ReadStorage<'a, Transform<V, V::M>>,
        WriteStorage<'a, BBall<V>>,
    );

    fn run(&mut self, (read_shape, read_transform, mut write_bball): Self::SystemData) {
        self.0.gather_events(read_shape.channel());
        for (_, shape, transform, bball) in (
            &self.0.modified,
            &read_shape,
            &read_transform,
            &mut write_bball,
        )
            .join()
        {
            *bball = BBall::new(&shape.verts, transform.pos);
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.0.reader_id = Some(WriteStorage::<Shape<V>>::fetch(world).register_reader());
    }
}

// TODO: similar to above
/// update bbox whenever shape is accessed mutably
#[derive(Default)]
pub struct UpdateBBoxSystem<V>(pub ModSystem<V>);

impl<'a, V: VectorTrait + Componentable> System<'a> for UpdateBBoxSystem<V> {
    type SystemData = (ReadStorage<'a, Shape<V>>, WriteStorage<'a, BBox<V>>);

    fn run(&mut self, (read_shape, mut write_bbox): Self::SystemData) {
        self.0.gather_events(read_shape.channel());
        for (_, shape, bbox) in (&self.0.modified, &read_shape, &mut write_bbox).join() {
            *bbox = shape.calc_bbox();
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.0.reader_id = Some(WriteStorage::<Shape<V>>::fetch(world).register_reader());
    }
}

/// resets static separators when shape is mutated
#[derive(Default)]
pub struct UpdateStaticClippingSystem<V>(pub ModSystem<V>);

impl<'a, V> System<'a> for UpdateStaticClippingSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadStorage<'a, Shape<V>>,
        WriteStorage<'a, ShapeClipState<V>>,
        Entities<'a>,
    );

    fn run(&mut self, (read_shape, mut write_shape_clip_state, entities): Self::SystemData) {
        // TODO: update spatial hash of updated shapes
        // clear static separators for shape, which will be repopulated next draw
        // still some odd clipping behavior from single faces, but this might have nothing
        // to do with updating

        // TODO: This is being triggered all the time for (at least) selected shapes. disabled for now
        self.0.gather_events(read_shape.channel());
        let mut entities_to_update = Vec::new();
        for (_, shape_clip_state, entity) in
            (&self.0.modified, &mut write_shape_clip_state, &entities).join()
        {
            shape_clip_state.separators = HashMap::new();
            shape_clip_state.in_front = HashSet::new();
            entities_to_update.push(entity);
        }
        // clear separators within other shapes
        // this is pretty awkwardly asymmetric, and maybe a single hashmap over tuples would make more sense for separators, but at some
        // point I switched away from that and i don't remember why
        for (shape_clip_state,) in (&mut write_shape_clip_state,).join() {
            for e in entities_to_update.iter() {
                shape_clip_state.remove(e);
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.0.reader_id = Some(WriteStorage::<Shape<V>>::fetch(world).register_reader());
    }
}

/// Updates shape components whenever the transform component is accessed mutably
#[derive(Default)]
pub struct TransformShapeSystem<V>(pub ModSystem<V>);

impl<'a, V> System<'a> for TransformShapeSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadExpect<'a, RefShapes<V>>,
        ReadStorage<'a, ShapeLabel>,
        ReadStorage<'a, Transform<V, V::M>>,
        WriteStorage<'a, Shape<V>>,
    );

    fn run(
        &mut self,
        (ref_shape, read_shape_label, read_transform, mut write_shape): Self::SystemData,
    ) {
        self.0.gather_events(read_transform.channel());
        for (_, transform, shape, shape_label) in (
            &self.0.modified,
            &read_transform,
            &mut write_shape,
            &read_shape_label,
        )
            .join()
        {
            shape.update_from_ref(
                ref_shape
                    .get(shape_label)
                    .unwrap_or_else(|| panic!("No ref shape with label {}", &shape_label.0)),
                transform,
            );
            if let ShapeType::SingleFace(ref mut single_face) = &mut shape.shape_type {
                single_face.update(&shape.verts, &shape.faces)
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.0.reader_id = Some(WriteStorage::<Transform<V, V::M>>::fetch(world).register_reader());
    }
}
