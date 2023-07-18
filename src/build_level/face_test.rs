use crate::draw::FaceTexture;
use crate::geometry::shape::buildshapes::{convex_shape_to_face_shape, ShapeBuilder};
use crate::graphics::colors::*;
use crate::vector::VecIndex;
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

fn build_wall<V: VectorTrait>(
    pos: V,
    rot: (VecIndex, VecIndex, Field),
    color: Color,
) -> impl Fn(ShapeEntityBuilderV<V>) -> ShapeEntityBuilderV<V> {
    move |builder| {
        builder
            .with_translation(pos)
            .with_rotation(rot.0, rot.1, rot.2)
            .with_color(color)
            .with_collider(Some(StaticCollider))
    }
}

fn build_test_walls<'a, V: VectorTrait + 'a>(
    build_shape: &'a ShapeEntityBuilderV<V>,
) -> impl IntoIterator<Item = ShapeEntityBuilderV<V>> + 'a {
    let theta = PI / 6.0;
    let cos = theta.cos();
    let sin = theta.sin();

    let floor = build_shape
        .clone()
        .with_translation(-V::one_hot(1) * 1.0)
        .with_rotation(-1, 1, PI / 2.)
        .with_color(BLUE)
        .with_collider(Some(StaticCollider));

    let walls = [
        build_wall(
            V::one_hot(-1) * (-1.0 - cos) + V::one_hot(1) * (sin - 1.0),
            (-1, 1, PI / 2.0 - theta),
            RED,
        ),
        build_wall(V::one_hot(-1) * 1.0, (0, -1, PI), GREEN),
        build_wall(V::one_hot(0) * 1.0, (0, -1, PI / 2.), ORANGE),
        build_wall(-V::one_hot(0) * 0.9 + V::one_hot(1) * 1.0, (0, -1, 3.0 * PI / 2.), CYAN),
    ];
    let floors = [
        floor.clone().with_translation(-V::one_hot(0) * 2.0),
        floor
            .clone()
            .with_translation(-V::one_hot(0) * 2.0 - V::one_hot(-1) * 2.0),
        floor
            .clone()
            .with_translation(V::one_hot(1) * (2.0 * sin) - V::one_hot(-1) * (2.0 + 2.0 * cos)),
        floor
            .clone()
            .with_translation(V::one_hot(1) * (2.0 * sin) - V::one_hot(-1) * (4.0 + 2.0 * cos))
            .with_color(MAGENTA),
        floor,
        build_shape
            .clone()
            .with_translation(V::one_hot(1) * 1.0)
            .with_rotation(-1, 1, -PI / 2.)
            .with_color(YELLOW),
    ];

    walls
        .into_iter()
        .map(|f| f(build_shape.clone()))
        .chain(floors)
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
    build_test_walls(&build_shape).into_iter().for_each(|b| {
        b.build(world).build();
    });
}
