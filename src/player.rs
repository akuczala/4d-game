use crate::camera::Camera;
use crate::collide::BBox;
use crate::collide::MoveNext;
use crate::components::*;
use crate::constants::MAX_TARGET_DIST;
use crate::ecs_utils::Componentable;
use crate::geometry::shape::buildshapes::ShapeBuilder;
use crate::geometry::transform::Scaling;
use crate::geometry::Line;
use crate::vector::MatrixTrait;
use crate::vector::{Field, VectorTrait};
use specs::prelude::*;
use specs::{Component, HashMapStorage};
use std::marker::PhantomData;

pub struct Player(pub Entity); //specifies entity of player

pub fn build_player<V>(world: &mut World, transform: &Transform<V, V::M>, heading: Option<Heading<V::M>>)
where
    V: VectorTrait + Componentable,
    V::M: Componentable + Clone,
{
    let camera = Camera::new(transform);
    let player_entity = world
        .create_entity()
        .with(*transform)
        .with(heading.unwrap_or(Heading(V::M::id())))
        .with(BBox {
            min: V::ones() * (-0.1) + transform.pos,
            max: V::ones() * (0.1) + transform.pos,
        })
        .with(camera) 
        .with(MoveNext::<V>::default())
        .with(MaybeTarget::<V>(None))
        .with(MaybeSelected(None))
        .build();

    world.insert(Player(player_entity));
}

// this may be a temp solution until we split the camera + player into separate entities
// e.g. the player has transform = heading, camera has transform where player is looking. would be 
// nice to have a parent relationship between the two transforms a la unity
// I wanted to be able to impl a method that returns M[-1], but it doesn't work because MatrixTrait has a free generic parameter V
pub struct Heading<M>(pub M);

pub struct ShapeTargetingSystem<V>(pub PhantomData<V>);

impl<'a, V> System<'a> for ShapeTargetingSystem<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    type SystemData = (
        ReadExpect<'a, Player>,
        ReadStorage<'a, Transform<V, V::M>>,
        ReadStorage<'a, Shape<V>>,
        ReadStorage<'a, ShapeType<V>>,
        ReadStorage<'a, ShapeClipState<V>>,
        Entities<'a>,
        WriteStorage<'a, MaybeTarget<V>>,
    );

    fn run(
        &mut self,
        (player, transforms, shapes, shape_types, shape_clip_state, entities, mut targets) : Self::SystemData,
    ) {
        let transform = transforms.get(player.0).expect("Player has no transform");
        let target = shape_targeting(
            transform,
            (&shapes, &shape_types, &shape_clip_state, &*entities).join(),
        ); //filter by shapes having a clip state
        *targets.get_mut(player.0).expect("Player has no target") = target;
    }
}
#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Cursor;

pub struct MaybeTarget<V>(pub Option<Target<V>>);

#[derive(Component)]
pub struct MaybeSelected(pub Option<Selected>);

pub struct Selected {
    pub entity: Entity,
    //pub selection_box_shape: Shape<V>,
}
impl Selected {
    pub fn new(entity: Entity) -> Self {
        Selected { entity }
    }
}

pub struct Target<V> {
    pub entity: Entity,
    pub distance: Field,
    pub point: V,
    //pub all_points : Vec<V>,
}

fn shape_targeting<'a, V: VectorTrait + 'a, I>(
    transform: &Transform<V, V::M>,
    iter: I,
) -> MaybeTarget<V>
where
    //for<'a> &'a I: std::iter::Iterator<Item=(&'a Shape<V>, &'a ShapeType<V>,&'a ShapeClipState<V>, Entity)>
    I: std::iter::Iterator<
        Item = (
            &'a Shape<V>,
            &'a ShapeType<V>,
            &'a ShapeClipState<V>,
            Entity,
        ),
    >,
{
    let pos = transform.pos;
    let dir = transform.frame[-1];
    let ray = Line(pos, pos + dir * MAX_TARGET_DIST);

    //loop through all shapes and check for nearest intersection
    let mut closest: Option<(Entity, Field, V)> = None;
    let mut all_points = Vec::<V>::new();
    for (shape, shape_type, shape_clip_state, e) in iter {
        for intersect_point in
            shape_type.line_intersect(shape, &ray, true, &shape_clip_state.face_visibility)
        {
            //find intersections of ray with visible faces
            all_points.push(intersect_point);
            let distsq = (intersect_point - pos).norm_sq();
            closest = match closest {
                Some((_cle, cldistsq, _clpoint)) => match distsq < cldistsq {
                    true => Some((e, distsq, intersect_point)),
                    false => closest,
                },
                None => Some((e, distsq, intersect_point)),
            }
        }
    }
    match closest {
        Some((e, distsq, point)) => MaybeTarget(Some(Target {
            entity: e,
            distance: distsq.sqrt(),
            point,
        })),
        None => MaybeTarget(None),
    }
}
