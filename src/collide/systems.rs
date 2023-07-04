use std::marker::PhantomData;

use ::specs::prelude::*;

use crate::{
    collide::get_entities_in_bbox,
    components::*,
    ecs_utils::{Componentable, ModSystem, SystemName},
    input::{key_map::PRINT_DEBUG, Input},
    spatial_hash::{HashInt, SpatialHashSet},
    vector::VectorTrait,
};

use super::{
    check_player_static_collisions, get_bbox_cells, get_dcoords_dcells, insert_static_bboxes,
    move_player, update_player_bbox, update_static_bboxes,
};

pub struct MovePlayerSystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for MovePlayerSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable + Clone,
{
    type SystemData = (
        ReadExpect<'a, Player>,
        WriteStorage<'a, MoveNext<V>>,
        WriteStorage<'a, Transform<V, V::M>>,
        WriteStorage<'a, Camera<V, V::M>>,
    );

    fn run(
        &mut self,
        (player, mut write_move_next, mut transforms, mut cameras): Self::SystemData,
    ) {
        move_player(
            write_move_next.get_mut(player.0).unwrap(),
            transforms.get_mut(player.0).unwrap(),
            cameras.get_mut(player.0).unwrap(),
        )
    }
}

//enter each statically colliding entity into every cell containing its bbox volume (either 1, 2, 4 ... up to 2^d cells)
//assuming that cells are large enough that all bboxes can fit in a cell
//for static objects, it is cheap to hash the volume since we need only do it once
//note that at present, we are rehashing ALL mutably accessed bboxes every step
// this occurs a) every time the player moves, and b) every time a shape is transformed
// so this is particularly inefficient for fixed pos spinning coins, which really don't need their hash updated
pub struct BBoxHashingSystem<V>(pub ModSystem<V>);

impl<'a, V: VectorTrait + Componentable> System<'a> for BBoxHashingSystem<V> {
    type SystemData = (
        ReadStorage<'a, BBox<V>>,
        Entities<'a>,
        WriteExpect<'a, SpatialHashSet<V, Entity>>,
    );

    fn run(&mut self, (read_bbox, entities, mut write_hash): Self::SystemData) {
        self.0.gather_events(read_bbox.channel());
        update_static_bboxes(
            &mut write_hash,
            (&read_bbox, &*entities, self.0.modified_or_inserted())
                .join()
                .map(|(b, e, _)| (b, e)),
        )
    }
    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.0.reader_id = Some(WriteStorage::<BBox<V>>::fetch(world).register_reader());
    }
}
impl SystemName for BBoxHashingSystem<()> {
    const NAME: &'static str = "bbox_hashing";
}

//add an update_bbox marker
pub struct UpdatePlayerBBox<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for UpdatePlayerBBox<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadExpect<'a, Player>,
        WriteStorage<'a, BBox<V>>,
        ReadStorage<'a, Transform<V, V::M>>,
    );

    fn run(&mut self, (player, mut write_bbox, transform): Self::SystemData) {
        update_player_bbox(
            write_bbox.get_mut(player.0).unwrap(),
            transform.get(player.0).unwrap().pos,
        )
    }
}

//stop movement through entities indexed in spatial hash set
//need only run these systems when the player is moving
pub struct PlayerCollisionDetectionSystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for PlayerCollisionDetectionSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadExpect<'a, Player>,
        ReadStorage<'a, BBox<V>>,
        WriteStorage<'a, InPlayerCell>,
        ReadExpect<'a, SpatialHashSet<V, Entity>>,
    );

    fn run(&mut self, (player, bbox, mut in_cell, hash): Self::SystemData) {
        in_cell.clear(); //clear previously marked
                         //maybe we should use the anticipated player bbox here
        let entities_in_bbox = get_entities_in_bbox(bbox.get(player.0).unwrap(), &hash);
        for &e in entities_in_bbox.iter() {
            in_cell
                .insert(e, InPlayerCell)
                .expect("PlayerCollisionDetectionSystem: entity in spatial hash doesn't exist");
        }
    }
}

pub struct PlayerStaticCollisionSystem<V>(pub PhantomData<V>);
impl<'a, V> System<'a> for PlayerStaticCollisionSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadExpect<'a, Player>,
        ReadStorage<'a, Transform<V, V::M>>,
        ReadStorage<'a, Shape<V>>,
        ReadStorage<'a, ShapeType<V>>,
        ReadStorage<'a, StaticCollider>,
        ReadStorage<'a, InPlayerCell>,
        WriteStorage<'a, MoveNext<V>>,
    );

    fn run(
        &mut self,
        (
            player,
            transform,
            shape,
            shape_types,
            static_collider,
            in_cell,
            mut write_move_next,
        ) : Self::SystemData,
    ) {
        check_player_static_collisions(
            write_move_next.get_mut(player.0).unwrap(),
            transform.get(player.0).unwrap().pos,
            (&shape, &shape_types, &static_collider, &in_cell)
                .join()
                .map(|(shape, shape_type, _, _)| (shape, shape_type)),
        )
    }
}

//print entities in the same cell as the player's bbox
pub struct CollisionTestSystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for CollisionTestSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadExpect<'a, Input>,
        ReadExpect<'a, Player>,
        ReadStorage<'a, Transform<V, V::M>>,
        ReadStorage<'a, Shape<V>>,
        ReadStorage<'a, ShapeType<V>>,
        ReadStorage<'a, BBox<V>>,
        ReadExpect<'a, SpatialHashSet<V, Entity>>,
    );

    fn run(
        &mut self,
        (input, player, transform, shapes, shape_types, bbox, hash): Self::SystemData,
    ) {
        use glium::glutin::event::VirtualKeyCode as VKC;
        if input.helper.key_released(PRINT_DEBUG) {
            //let mut out_string = "Entities: ".to_string();
            let entities_in_bbox = get_entities_in_bbox(bbox.get(player.0).unwrap(), &hash);
            let player_pos = transform.get(player.0).unwrap().pos;
            if entities_in_bbox
                .iter()
                .any(|&e| match shape_types.get(e).unwrap() {
                    ShapeType::Convex(_convex) => {
                        Convex::point_within(player_pos, 0.1, &shapes.get(e).unwrap().faces)
                    }
                    _ => false,
                })
            {
                println!("in thing")
            } else {
                println!("not in thing")
            }
            // for e in entities_in_bbox {
            // 	out_string = format!("{} {},", out_string, e.id())
            // }
            // println!("{}", out_string);
        }
    }
}
