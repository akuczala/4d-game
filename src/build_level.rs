mod face_test;
mod fun_level;
mod invert_test;
mod level_1;

use crate::coin::Coin;
use crate::collide::StaticCollider;
use crate::components::{Cursor, Heading, Player, Transform};
use crate::config::{Config, LevelConfig};
use crate::constants::COIN_TEXTURE_LABEL_STR;
use crate::draw::draw_line_collection::DrawLineCollection;
use crate::draw::texture::ShapeTextureBuilder;
use crate::draw::visual_aids::{calc_grid_lines, draw_horizon, draw_sky, draw_stars};
use crate::ecs_utils::Componentable;
use crate::geometry::shape::buildshapes::ShapeBuilder;
use crate::geometry::shape::{build_shape_library, RefShapes};
use crate::graphics::colors::*;
use crate::saveload::load_level_from_file;
use crate::shape_entity_builder::ShapeEntityBuilderV;
use crate::vector::VectorTrait;
use serde::de::DeserializeOwned;
use specs::prelude::*;

use self::face_test::build_test_level;
use self::fun_level::build_fun_level;
use self::invert_test::build_inverted_test_level;
use self::level_1::build_lvl_1;

pub fn insert_static_collider<V>(
    world: &mut World,
    ref_shapes: &RefShapes<V>,
    shape_builder: ShapeEntityBuilderV<V>,
) where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    shape_builder
        .with_collider(Some(StaticCollider))
        .build(ref_shapes, world)
        .build();
}
pub fn insert_coin<V>(
    world: &mut World,
    ref_shapes: &RefShapes<V>,
    shape_builder: ShapeEntityBuilderV<V>,
) where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    shape_builder
        .with_texture(ShapeTextureBuilder::from_resource(
            COIN_TEXTURE_LABEL_STR.into(),
        ))
        .with_coin(Some(Coin))
        .build(ref_shapes, world)
        .build();
}

pub fn build_scene<V>(world: &mut World)
where
    V: VectorTrait + Componentable + DeserializeOwned,
    V::SubV: Componentable + DeserializeOwned,
    V::M: Componentable + DeserializeOwned,
{
    let ref_shapes = build_shape_library::<V>();
    build_level(&ref_shapes, world);
    world.insert(ref_shapes);
}

pub fn build_level<V: VectorTrait>(ref_shapes: &RefShapes<V>, world: &mut World)
where
    V: VectorTrait + Componentable + DeserializeOwned,
    V::SubV: Componentable + DeserializeOwned,
    V::M: Componentable + DeserializeOwned,
{
    let config: Config = (*world.read_resource::<Config>()).clone();
    match config.scene.level {
        LevelConfig::Level1 => build_lvl_1(
            world,
            ref_shapes,
            config.scene.level_1.unwrap_or_default().open_center,
        ),
        LevelConfig::Test1 => build_test_level(world, ref_shapes),
        LevelConfig::Test2 => {
            build_fun_level()
                .into_iter()
                .for_each(|b| insert_static_collider(world, ref_shapes, b));
        }
        LevelConfig::Test3 => build_inverted_test_level(ref_shapes, world),
        LevelConfig::Load => config
            .scene
            .load_config(V::DIM.into())
            .as_ref()
            .ok_or(())
            .map_err(|_| println!("No load path specified"))
            .map(|load| load_level_from_file(&load.path, ref_shapes, world).unwrap_or_default())
            .unwrap_or_default(),
        LevelConfig::Empty => (),
    };
    // Default player placement
    if world.try_fetch::<Player>().is_none() {
        init_player::<V>(world, None, None);
    }
    build_scenery::<V>(world);
}

pub fn build_scenery<V: VectorTrait + Componentable>(world: &mut World) {
    let config: Config = (*world.read_resource::<Config>()).clone();
    let scene_config = config.scene;
    vec![
        DrawLineCollection::from_lines(
            calc_grid_lines(V::one_hot(1) * (-1.0) + (V::ones() * 0.5), 1.0, 2),
            WHITE.set_alpha(0.2),
        ),
        DrawLineCollection(draw_sky::<V>(config.draw.fuzz_lines.sky_num)),
        DrawLineCollection::from_lines(
            draw_horizon::<V>(config.draw.fuzz_lines.horizon_num),
            ORANGE.set_alpha(0.5),
        ),
        DrawLineCollection(draw_stars::<V>()),
    ]
    .into_iter()
    .zip([
        scene_config.grid,
        scene_config.sky,
        scene_config.horizon,
        scene_config.stars,
    ])
    .for_each(|(dlc, enabled)| {
        if enabled {
            world.create_entity().with(dlc).build();
        }
    });
}

pub fn init_player<V>(
    world: &mut World,
    transform: Option<Transform<V, V::M>>,
    heading: Option<Heading<V::M>>,
) where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    crate::player::build_player(world, transform.unwrap_or_default(), heading);
    init_cursor::<V>(world);
}

pub fn init_cursor<V>(world: &mut World)
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
{
    world
        .create_entity()
        .with(Cursor)
        .with(ShapeBuilder::<V::SubV>::build_cube(0.03).build())
        .build();
}
