use std::path::Path;

use specs::prelude::*;

use crate::{
    build_level::build_level,
    components::{Player, StaticCollider, Transform},
    config::{Config, LevelConfig},
    geometry::shape::build_shape_library,
    saveload::{load_level_from_file, save_level_to_file},
    tests::new_world,
    vector::{Vec3, VectorTrait},
};

type V = Vec3;

fn move_player(world: &mut World, transform: Transform<V, <V as VectorTrait>::M>) {
    let mut transforms = world.write_storage::<Transform<V, <V as VectorTrait>::M>>();
    let player_transform = transforms.get_mut(world.fetch::<Player>().0).unwrap();
    player_transform.set_transform(transform);
}

fn get_player_transform(world: &World) -> Transform<V, <V as VectorTrait>::M> {
    let transforms: Storage<
        '_,
        Transform<Vec3, crate::vector::Mat3>,
        specs::shred::Fetch<
            '_,
            specs::storage::MaskedStorage<Transform<Vec3, crate::vector::Mat3>>,
        >,
    > = world.read_storage::<Transform<V, <V as VectorTrait>::M>>();
    *transforms.get(world.fetch::<Player>().0).unwrap()
}

#[test]
fn test_saveload() {
    let mut world = new_world::<V>();
    let ref_shapes = build_shape_library::<V>();

    let mut config = world.fetch_mut::<Config>();
    config.scene.level = LevelConfig::Test1;
    drop(config);

    build_level(&ref_shapes, &mut world);
    let player_pos = V::one_hot(0);
    move_player(&mut world, Transform::pos(player_pos));
    save_level_to_file::<V>(Path::new("./tmp.3d.ron"), &mut world).unwrap();
    let initial_count = world.read_component::<StaticCollider>().count();

    let mut deserialized_world = new_world::<V>();
    let ref_shapes = build_shape_library::<V>();

    load_level_from_file("./tmp.3d.ron", &ref_shapes, &mut deserialized_world).unwrap();
    let final_count = deserialized_world
        .read_component::<StaticCollider>()
        .count();
    assert_eq!(initial_count, final_count);
    assert!(V::is_close(get_player_transform(&world).pos, player_pos));
}
