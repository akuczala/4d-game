use rand::random;
#[cfg(test)]
use specs::{World, WorldExt};

use crate::{
    components::Transform,
    config::load_config,
    constants::TWO_PI,
    ecs_utils::Componentable,
    engine::get_engine_dispatcher_builder,
    geometry::transform::Scaling,
    vector::{random_sphere_point, rotation_matrix, Field, VectorTrait},
};

mod test_boundaries;
mod test_saveload;
mod test_single_face;
mod test_transform;
pub mod utils;

pub fn new_world<V>() -> World
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
    V::SubV: Componentable,
{
    let mut world = World::new();
    world.insert(load_config());
    let mut dispatcher = get_engine_dispatcher_builder::<V>().build();
    dispatcher.setup(&mut world);
    world
}

pub fn random_vec<V: VectorTrait>() -> V {
    random_sphere_point::<V>() * (0.01 + random::<Field>())
}

pub fn random_rotation_matrix<V: VectorTrait>() -> V::M {
    rotation_matrix(
        random_vec::<V>(),
        random_vec(),
        Some(random::<Field>() * TWO_PI),
    )
}

pub fn random_scaling<V: VectorTrait>() -> Scaling<V> {
    Scaling::Vector(V::random()) // positive only for now
}

pub fn random_transform<V: VectorTrait>() -> Transform<V, V::M> {
    Transform::new(
        Some(random_vec()),
        Some(random_rotation_matrix::<V>()),
        Some(random_scaling()),
    )
}
