use std::collections::{HashMap, HashSet};

use specs::{ReadStorage, WriteStorage, World, System};
use specs::prelude::*;

use crate::components::{BBox, HasBBox, ShapeLabel, ShapeType, ShapeClipState, MaybeSelected, Player};
use crate::ecs_utils::Componentable;
use crate::geometry::shape::RefShapes;
use crate::vector::MatrixTrait;
use crate::{vector::VectorTrait, ecs_utils::ModSystem, components::{Shape, Transform, BBall}};

//TODO: we don't always need to update all of this when a shape gets mutated - e.g. spinning coins do not change any of these components.
// maybe we can include a marker or something to indicate that these shapes should be excluded?
// or something more sophisticated where we indicate when pos and/or rotation have been updated?
#[derive(Default)]
pub struct UpdateBBallSystem<V>(pub ModSystem<V>);


impl<'a, V, M> System<'a> for UpdateBBallSystem<V>
where
        V: VectorTrait<M=M> + Componentable,
        M: MatrixTrait<V> + Componentable
{

    type SystemData = (
        ReadStorage<'a, Shape<V>>,
        ReadStorage<'a, Transform<V, V::M>>,
        WriteStorage<'a, BBall<V>>
    );

    fn run(
        &mut self, 
        (
            read_shape,
            read_transform,
            mut write_bball
        ): Self::SystemData
    ) {
        self.0.gather_events(read_shape.channel());
        for (_, shape, transform, bball) in (&self.0.modified, &read_shape, &read_transform, &mut write_bball).join() {
            *bball =  BBall::new(&shape.verts, transform.pos);
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.0.reader_id = Some(
            WriteStorage::<Shape<V>>::fetch(&world).register_reader()
        );
    }
}

#[derive(Default)]
pub struct UpdateBBoxSystem<V>(pub ModSystem<V>);

impl<'a,V: VectorTrait + Componentable> System<'a> for UpdateBBoxSystem<V> {

	type SystemData = (
		ReadStorage<'a, Shape<V>>,
		WriteStorage<'a, BBox<V>>
	);

	fn run(
        &mut self, (
            read_shape,
            mut write_bbox
        ) : Self::SystemData
    ) {
        self.0.gather_events(read_shape.channel());
		for (_, shape, bbox) in (&self.0.modified, &read_shape, &mut write_bbox).join() {
			*bbox = shape.calc_bbox();
		}
	}

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.0.reader_id = Some(
            WriteStorage::<Shape<V>>::fetch(&world).register_reader()
        );
    }
}

// resets static separators when shape is mutated
#[derive(Default)]
pub struct UpdateStaticClippingSystem<V>(pub ModSystem<V>);

impl<'a, V, M> System<'a> for UpdateStaticClippingSystem<V>
where
        V: VectorTrait<M=M> + Componentable,
        M: MatrixTrait<V> + Componentable
{
    type SystemData = (
        ReadStorage<'a, Shape<V>>,
        WriteStorage<'a, ShapeClipState<V>>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (
            read_shape,
            mut write_shape_clip_state,
            entities
        ): Self::SystemData) {
        // TODO: update spatial hash of updated shapes
        // clear static separators for shape, which will be repopulated next draw
        // still some odd clipping behavior from single faces, but this might have nothing
        // to do with updating
        self.0.gather_events(read_shape.channel());
        let mut entities_to_update = Vec::new();
        for (_, shape_clip_state, entity) in (&self.0.modified, &mut write_shape_clip_state, &entities).join() {
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
        self.0.reader_id = Some(
            WriteStorage::<Shape<V>>::fetch(&world).register_reader()
        );
    }
}

#[derive(Default)]
pub struct TransformShapeSystem<V>(pub ModSystem<V>);

impl<'a, V, M> System<'a> for TransformShapeSystem<V>
    where
        V: VectorTrait<M=M> + Componentable,
        M: MatrixTrait<V> + Componentable
{

	type SystemData = (
        ReadExpect<'a,RefShapes<V>>,
        ReadStorage<'a,ShapeLabel>,
		ReadStorage<'a, Transform<V, M>>,
		WriteStorage<'a, Shape<V>>,
        WriteStorage<'a, ShapeType<V>>,
	);

	fn run(
        &mut self, (
            ref_shape,
            read_shape_label,
            read_transform,
            mut write_shape,
            mut write_shape_type,
        ) : Self::SystemData
    ) {
        self.0.gather_events(read_transform.channel());
		for (_, transform, shape, shape_label, shape_type) in (&self.0.modified, &read_transform, &mut write_shape, &read_shape_label, &mut write_shape_type).join() {
			shape.update_from_ref(
                ref_shape.get(shape_label)
                .expect(&format!("No ref shape with label {}", &shape_label.0)),
                transform
            );
            if let ShapeType::SingleFace(single_face) = shape_type {
                single_face.update(&shape)
            }
		}
	}

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.0.reader_id = Some(
            WriteStorage::<Transform<V, V::M>>::fetch(&world).register_reader()
        );
    }
}
