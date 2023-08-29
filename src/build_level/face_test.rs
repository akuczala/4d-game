use crate::constants::TWO_SIDED_FACE_LABEL_STR;

use crate::draw::texture::texture_builder::TextureBuilder;

use crate::graphics::colors::*;
use crate::utils::ValidDimension;
use crate::vector::VecIndex;
use crate::{components::StaticCollider, constants::PI};

use specs::{Builder, World};

use crate::{
    components::{RefShapes, ShapeLabel, Transformable},
    ecs_utils::Componentable,
    geometry::transform::Scaling,
    graphics::colors::YELLOW,
    shape_entity_builder::{ShapeEntityBuilder, ShapeEntityBuilderV},
    vector::{Field, VectorTrait},
};

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
        build_wall(
            -V::one_hot(0) * 0.9 + V::one_hot(1) * 1.0,
            (0, -1, 3.0 * PI / 2.),
            CYAN,
        ),
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
        //TODO: yellow does not collide??
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
pub fn build_test_level<V>(world: &mut World, ref_shapes: &RefShapes<V>)
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    let n_divisions = match V::DIM.into() {
        ValidDimension::Three => vec![4, 4],
        ValidDimension::Four => vec![4, 4, 4],
    };
    let build_shape: ShapeEntityBuilderV<V> = ShapeEntityBuilder::new_from_ref_shape(
        ref_shapes,
        ShapeLabel::from_str(TWO_SIDED_FACE_LABEL_STR),
    )
    .with_scale(Scaling::Scalar(2.0))
    .with_face_texture(TextureBuilder::new().make_tile_texture(vec![0.8], n_divisions));
    build_test_walls(&build_shape).into_iter().for_each(|b| {
        b.build(world).build();
    });
}
