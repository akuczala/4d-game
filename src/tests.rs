#[cfg(test)]
use specs::{World, WorldExt};

use crate::{
    config::load_config, ecs_utils::Componentable, engine::get_engine_dispatcher_builder,
    vector::VectorTrait,
};

mod test_boundaries;
mod test_saveload;
mod test_single_face;
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
