use specs::{Builder, World};

use crate::{
    components::{RefShapes, ShapeLabel, StaticCollider, Transformable},
    constants::{CUBE_LABEL_STR, FUZZY_COLOR_CUBE_LABEL_STR, HALF_PI},
    draw::texture::ShapeTextureBuilder,
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
    let fuzzy_tex = ShapeTextureBuilder::from_resource(FUZZY_COLOR_CUBE_LABEL_STR.into());
    ShapeEntityBuilderV::new(ShapeLabel::from("OpenCube"))
        .with_scale(crate::geometry::transform::Scaling::Scalar(2.0))
        .with_texture(fuzzy_tex.clone())
        .with_collider(Some(StaticCollider))
        .with_translation(V::one_hot(0) * 4.0)
        .with_rotation(0, 1, HALF_PI)
        // TODO: When this angle is small (~0.2), we can sometimes go through the front (cyan) face
        .with_rotation(1, 2, 1.0)
        .build(ref_shapes, world)
        .build();

    ShapeEntityBuilderV::new(ShapeLabel::from("OpenInvertedCube"))
        .with_scale(crate::geometry::transform::Scaling::Scalar(2.0))
        .with_texture(fuzzy_tex.clone())
        .with_collider(Some(StaticCollider))
        .with_translation(V::one_hot(0) * 4.0 + V::one_hot(2) * 4.0)
        .with_rotation(0, 1, HALF_PI)
        // TODO: When this angle is small (~0.2), we can sometimes go through the front (cyan) face
        .with_rotation(1, 2, 1.0)
        .build(ref_shapes, world)
        .build();

    ShapeEntityBuilderV::new(ShapeLabel::from(CUBE_LABEL_STR))
        .with_translation(V::one_hot(0) * 8.0)
        .with_texture(fuzzy_tex.clone())
        .with_color(WHITE)
        .with_collider(Some(StaticCollider))
        .build(ref_shapes, world)
        .build();

    ShapeEntityBuilderV::new(ShapeLabel::from(CUBE_LABEL_STR))
        .with_texture(fuzzy_tex)
        .with_color(WHITE)
        .with_translation(V::one_hot(0) * 1.0)
        .with_collider(Some(StaticCollider))
        .build(ref_shapes, world)
        .build();
}
