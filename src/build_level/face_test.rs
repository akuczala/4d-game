use crate::draw::FaceTexture;
use crate::geometry::shape::buildshapes::{convex_shape_to_face_shape, ShapeBuilder};
use crate::graphics::colors::*;
use crate::{components::StaticCollider, constants::PI};

use specs::{Builder, World, WorldExt};

use crate::{
    components::{RefShapes, Shape, ShapeLabel, ShapeTexture, Transformable},
    config::Config,
    constants::{COIN_LABEL_STR, CUBE_LABEL_STR, FACE_SCALE},
    draw::{
        self,
        texture::{color_cube_texture, fuzzy_color_cube_texture},
    },
    ecs_utils::Componentable,
    geometry::transform::Scaling,
    graphics::colors::YELLOW,
    shape_entity_builder::{ShapeEntityBuilder, ShapeEntityBuilderV},
    vector::{Field, VectorTrait},
};

use super::{insert_coin, insert_static_collider};

fn build_test_walls<V>(build_shape: &ShapeEntityBuilderV<V>, world: &mut World)
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    let theta = PI / 6.0;
    let cos = theta.cos();
    let sin = theta.sin();
    build_shape
        .clone()
        .with_translation(V::one_hot(-1) * (-1.0 - cos) + V::one_hot(1) * (sin - 1.0))
        .with_rotation(-1, 1, PI / 2.0 - theta)
        .with_color(RED)
        .build(world)
        .with(StaticCollider)
        .build();
    build_shape
        .clone()
        .with_translation(V::one_hot(-1) * 1.0)
        .with_rotation(0, -1, PI)
        .with_color(GREEN)
        .build(world)
        .with(StaticCollider)
        .build();
    build_shape
        .clone()
        .with_translation(V::one_hot(0) * 1.0)
        .with_rotation(0, -1, PI / 2.)
        .with_color(ORANGE)
        .build(world)
        .with(StaticCollider)
        .build();
    build_shape
        .clone()
        .with_translation(-V::one_hot(0) * 1.0)
        .with_rotation(0, -1, 3.0 * PI / 2.)
        .with_color(CYAN)
        .build(world)
        .with(StaticCollider)
        .build();
    let floor = build_shape
        .clone()
        .with_translation(-V::one_hot(1) * 1.0)
        .with_rotation(-1, 1, PI / 2.)
        .with_color(BLUE);
    floor
        .clone()
        .with_translation(-V::one_hot(0) * 2.0)
        .build(world)
        .with(StaticCollider)
        .build();
    floor
        .clone()
        .with_translation(-V::one_hot(0) * 2.0 - V::one_hot(-1) * 2.0)
        .build(world)
        .with(StaticCollider)
        .build();
    floor
        .clone()
        .with_translation(V::one_hot(1) * (2.0 * sin) - V::one_hot(-1) * (2.0 + 2.0 * cos))
        .build(world)
        .with(StaticCollider)
        .build();
    floor
        .clone()
        .with_translation(V::one_hot(1) * (2.0 * sin) - V::one_hot(-1) * (4.0 + 2.0 * cos))
        .with_color(MAGENTA)
        .build(world)
        .with(StaticCollider)
        .build();
    floor.build(world).with(StaticCollider).build();
    build_shape
        .clone()
        .with_translation(V::one_hot(1) * 1.0)
        .with_rotation(-1, 1, -PI / 2.)
        .with_color(YELLOW)
        .build(world)
        .with(StaticCollider)
        .build();
}
pub fn build_test_level<V>(world: &mut World, ref_shapes: &mut RefShapes<V>)
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    let (n_divisions, frame_vertis) = match V::DIM {
        3 => (vec![4, 4], vec![1, 3]),
        4 => (vec![4, 4, 4], vec![1, 3, 4]),
        _ => panic!("Cannot build test level in {} dimensions.", { V::DIM }),
    };
    let sub_wall = ShapeBuilder::<V::SubV>::build_cube(2.0).build();
    let wall_label = ShapeLabel("Wall".to_string());
    let (wall, wall_single_face) = convex_shape_to_face_shape(sub_wall, true);
    ref_shapes.insert(wall_label.clone(), wall);
    let build_shape: ShapeEntityBuilderV<V> =
        ShapeEntityBuilder::new_face_from_ref_shape(ref_shapes, wall_single_face, wall_label)
            .with_face_texture(FaceTexture {
                texture: draw::Texture::make_tile_texture(&[0.8], &n_divisions),
                texture_mapping: Some(draw::TextureMapping {
                    origin_verti: 0,
                    frame_vertis,
                }),
            });
    build_test_walls(&build_shape, world);
}
