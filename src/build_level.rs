mod face_test;
mod fun_level;
mod level_1;

use crate::coin::Coin;
use crate::collide::StaticCollider;
use crate::components::{Cursor, Transform};
use crate::config::{Config, FuzzLinesConfig, LevelConfig};
use crate::constants::{COIN_LABEL_STR, CUBE_LABEL_STR};
use crate::draw::draw_line_collection::DrawLineCollection;
use crate::draw::texture::{color_cube_texture, fuzzy_color_cube_texture};
use crate::draw::visual_aids::{calc_grid_lines, draw_horizon, draw_sky, draw_stars};
use crate::ecs_utils::Componentable;
use crate::geometry::shape::buildshapes::ShapeBuilder;
use crate::geometry::shape::{build_shape_library, RefShapes, ShapeLabel};
use crate::geometry::transform::{Scaling, Transformable};
use crate::geometry::Shape;
use crate::graphics::colors::*;
use crate::shape_entity_builder::ShapeEntityBuilderV;
use crate::vector::{Field, VectorTrait};
use specs::prelude::*;

use self::face_test::build_test_level;
use self::fun_level::build_fun_level;
use self::level_1::build_lvl_1;

pub fn insert_static_collider<V>(world: &mut World, shape_builder: ShapeEntityBuilderV<V>)
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    shape_builder.build(world).with(StaticCollider).build();
}
pub fn insert_coin<V>(world: &mut World, shape_builder: ShapeEntityBuilderV<V>)
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    shape_builder.build(world).with(Coin).build();
}

pub fn build_scene<V>(world: &mut World)
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    let mut ref_shapes = build_shape_library::<V>();
    build_level(&mut ref_shapes, world);
    world.insert(ref_shapes);
    init_player(world, V::zero());
}

pub fn build_level<V: VectorTrait>(ref_shapes: &mut RefShapes<V>, world: &mut World)
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
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
            build_fun_level(config.fuzz_lines, ref_shapes)
                .into_iter()
                .for_each(|b| insert_static_collider(world, b));
        }
        LevelConfig::Empty => (),
    };
    build_empty_level::<V>(world);
}

pub fn build_empty_level<V: VectorTrait + Componentable>(world: &mut World) {
    let config: Config = (*world.read_resource::<Config>()).clone();
    let scene_config = config.scene;
    vec![
        DrawLineCollection::from_lines(
            calc_grid_lines(V::one_hot(1) * (-1.0) + (V::ones() * 0.5), 1.0, 2),
            WHITE.set_alpha(0.2),
        ),
        DrawLineCollection(draw_sky::<V>(config.fuzz_lines.sky_num)),
        DrawLineCollection::from_lines(
            draw_horizon::<V>(config.fuzz_lines.horizon_num),
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

pub fn init_player<V>(world: &mut World, pos: V)
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    let transform = Transform::identity().with_translation(pos);
    crate::player::build_player(world, &transform, None);
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
