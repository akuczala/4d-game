use serde::{Deserialize, Serialize};
use specs::prelude::*;

use crate::{
    build_level::build_level,
    components::{Shape, StaticCollider},
    config::{load_config, Config, LevelConfig},
    engine::get_engine_dispatcher_builder,
    geometry::shape::build_shape_library,
    saveload::{load_level_from_file, save_level_to_file},
    tests::new_world,
    vector::Vec3,
};

type V = Vec3;

#[test]
fn test_saveload() {
    let mut world = new_world::<V>();
    let ref_shapes = build_shape_library::<V>();

    let mut config = world.fetch_mut::<Config>();
    config.scene.level = LevelConfig::Level1;
    drop(config);

    build_level(&ref_shapes, &mut world);
    save_level_to_file::<V>("./test_save_level.ron", &mut world).unwrap();
    let initial_count = world.read_component::<StaticCollider>().count();

    let mut deserialized_world = new_world::<V>();
    let ref_shapes = build_shape_library::<V>();
    load_level_from_file(
        "./test_save_level.ron",
        &ref_shapes,
        &mut deserialized_world,
    )
    .unwrap();
    let final_count = deserialized_world
        .read_component::<StaticCollider>()
        .count();
    assert_eq!(initial_count, final_count)
}
