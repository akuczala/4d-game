use specs::{Builder, World};

use crate::{
    components::{RefShapes, ShapeLabel, StaticCollider, Transformable},
    constants::{CUBE_LABEL_STR, HALF_PI, INVERTED_CUBE_LABEL_STR},
    draw::texture::{color_cube_texture, fuzzy_color_cube_texture},
    ecs_utils::Componentable,
    graphics::colors::WHITE,
    shape_entity_builder::ShapeEntityBuilderV,
    vector::VectorTrait,
};

pub fn build_inverted_test_level<V>(ref_shapes: &RefShapes<V>, world: &mut World)
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
    V::SubV: Componentable,
{
    ShapeEntityBuilderV::new_from_ref_shape(ref_shapes, ShapeLabel::from_str("OpenCube"))
        .with_scale(crate::geometry::transform::Scaling::Scalar(2.0))
        .with_texturing_fn(|shape| fuzzy_color_cube_texture(shape, 500))
        .with_collider(Some(StaticCollider))
        .with_translation(V::one_hot(0) * 4.0)
        .with_rotation(0, 1, HALF_PI)
        // TODO: When this angle is small (~0.2), we can sometimes go through the front (cyan) face
        .with_rotation(1, 2, 1.0)
        .build(world)
        .build();

    ShapeEntityBuilderV::new_from_ref_shape(ref_shapes, ShapeLabel::from_str(CUBE_LABEL_STR))
        .with_translation(V::one_hot(0) * 8.0)
        .with_texturing_fn(|shape| fuzzy_color_cube_texture(shape, 500))
        .with_color(WHITE)
        .with_collider(Some(StaticCollider))
        .build(world)
        .build();

    ShapeEntityBuilderV::new_from_ref_shape(ref_shapes, ShapeLabel::from_str(CUBE_LABEL_STR))
        .with_texturing_fn(|shape| fuzzy_color_cube_texture(shape, 500))
        .with_color(WHITE)
        .with_translation(V::one_hot(0) * 1.0)
        .with_collider(Some(StaticCollider))
        .build(world)
        .build();
}
