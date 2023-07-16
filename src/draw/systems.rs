use std::marker::PhantomData;

use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    components::{
        BBall, Camera, ClipState, Cursor, Player, Shape, ShapeClipState, ShapeType, Transform,
    },
    config::Config,
    ecs_utils::{Componentable, SystemName},
    vector::{Field, VectorTrait}, constants::FACE_SCALE,
};

use super::{
    calc_shapes_lines,
    clipping::{calc_in_front, calc_in_front_pair, InFrontArg},
    draw_cursor,
    draw_line_collection::{draw_collection, DrawLineCollection},
    transform_draw_line, update_shape_visibility, DrawLine, DrawLineList, ShapeTexture,
};

//would be nicer to move lines out of read_in_lines rather than clone them
pub struct TransformDrawLinesSystem<V>(pub PhantomData<V>);
impl<'a, V> System<'a> for TransformDrawLinesSystem<V>
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadExpect<'a, DrawLineList<V>>,
        WriteExpect<'a, DrawLineList<V::SubV>>,
        ReadStorage<'a, Camera<V>>,
        ReadStorage<'a, Transform<V, V::M>>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, Config>,
    );

    fn run(
        &mut self,
        (read_in_lines, mut write_out_lines, camera, transform, player, config): Self::SystemData,
    ) {
        //write new vec of draw lines to DrawLineList
        write_out_lines.0 = read_in_lines
            .0
            .iter()
            .flat_map(|line| {
                transform_draw_line(
                    line.clone(),
                    transform.get(player.0).unwrap(),
                    camera.get(player.0).unwrap(),
                    &config.view_config,
                )
            })
            .collect();
    }
}
impl SystemName for TransformDrawLinesSystem<()> {
    const NAME: &'static str = "transform_draw_lines";
}

pub struct DrawCursorSystem<V>(pub PhantomData<V>);
impl<'a, V> System<'a> for DrawCursorSystem<V>
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
{
    type SystemData = (
        ReadStorage<'a, Cursor>,
        ReadStorage<'a, Shape<V::SubV>>,
        WriteExpect<'a, DrawLineList<V::SubV>>,
    );

    fn run(&mut self, (cursors, shapes, mut draw_lines): Self::SystemData) {
        //write new vec of draw lines to DrawLineList
        for (_, shape) in (&cursors, &shapes).join() {
            draw_lines.0.extend(draw_cursor(shape));
        }
    }
}
impl SystemName for DrawCursorSystem<()> {
    const NAME: &'static str = "draw_cursor";
}

//in this implementation, the length of the vec is always
//the same, and invisible faces are just sequences of None
//seems to be significantly slower than not padding and just changing the buffer when needed
//either way, we need to modify the method to write to an existing line buffer rather than allocating new Vecs
pub struct VisibilitySystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for VisibilitySystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadStorage<'a, Shape<V>>,
        WriteStorage<'a, ShapeClipState<V>>,
        ReadStorage<'a, ShapeType<V>>,
        ReadStorage<'a, Transform<V, V::M>>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, ClipState<V>>,
    );

    fn run(
        &mut self,
        (
			shapes,
			mut shape_clip_states,
			shape_types,
			transform,
			player,
			clip_state
		) : Self::SystemData,
    ) {
        for (shape, shape_clip_state, shape_type) in
            (&shapes, &mut shape_clip_states, &shape_types).join()
        {
            update_shape_visibility(
                transform.get(player.0).unwrap().pos,
                shape,
                shape_clip_state,
                shape_type,
                &clip_state,
            )
        }
    }
}
impl SystemName for VisibilitySystem<()> {
    const NAME: &'static str = "visibility";
}

pub struct CalcShapesLinesSystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for CalcShapesLinesSystem<V>
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadStorage<'a, Shape<V>>,
        ReadStorage<'a, ShapeTexture<V::SubV>>,
        ReadStorage<'a, ShapeClipState<V>>,
        ReadExpect<'a, ClipState<V>>,
        WriteExpect<'a, DrawLineList<V>>, // TODO: break up into components so that these can be processed more in parallel with par_iter?
    );

    fn run(
        &mut self,
        (
		shapes,
		shape_textures,
		shape_clip_states,
		clip_state,
		mut lines
	) : Self::SystemData,
    ) {
        lines.0 = calc_shapes_lines(
            &shapes,
            &shape_textures,
            &shape_clip_states,
            &[FACE_SCALE],
            &clip_state,
        );
    }
}
impl SystemName for CalcShapesLinesSystem<()> {
    const NAME: &'static str = "calc_lines";
}

pub struct InFrontSystem<V>(pub PhantomData<V>);
impl<'a, V> System<'a> for InFrontSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadStorage<'a, Shape<V>>,
        ReadStorage<'a, BBall<V>>,
        WriteStorage<'a, ShapeClipState<V>>,
        Entities<'a>,
        ReadStorage<'a, Transform<V, V::M>>,
        ReadExpect<'a, Player>,
    );

    fn run(
        &mut self,
        (shape_data, bball_data, mut shape_clip_state,entities,transform,player) : Self::SystemData,
    ) {
        calc_in_front(
            &shape_data,
            &bball_data,
            &mut shape_clip_state,
            &entities,
            &transform.get(player.0).unwrap().pos,
        );
    }
}
impl SystemName for InFrontSystem<()> {
    const NAME: &'static str = "in_front";
}

pub struct DrawLineCollectionSystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for DrawLineCollectionSystem<V>
where
    V: VectorTrait + Componentable,
{
    type SystemData = (
        ReadStorage<'a, DrawLineCollection<V>>,
        ReadStorage<'a, ShapeClipState<V>>,
        ReadExpect<'a, ClipState<V>>,
        WriteExpect<'a, DrawLineList<V>>, // TODO: break up into components so that these can be processed more in parallel with par_iter?
    );

    // TODO: this will clip using ALL shapes. is there a way to reduce the workload?
    fn run(
        &mut self,
        (line_collection_storage, read_shape_clip_state, clip_state, mut lines): Self::SystemData,
    ) {
        for lines_coll in line_collection_storage.join() {
            lines.0.extend(draw_collection(
                lines_coll,
                clip_state
                    .clipping_enabled
                    .then_some((&read_shape_clip_state).join()),
            ));
        }
    }
}
impl SystemName for DrawLineCollectionSystem<()> {
    const NAME: &'static str = "line_collection_system";
}
