use specs::{Builder, World};

use crate::{
    components::{RefShapes, ShapeLabel, StaticCollider, Transformable},
    constants::{CUBE_LABEL_STR, INVERTED_CUBE_LABEL_STR},
    draw::texture::color_cube_texture,
    ecs_utils::Componentable,
    shape_entity_builder::ShapeEntityBuilderV,
    vector::VectorTrait,
};

pub fn build_inverted_test_level<V>(ref_shapes: &RefShapes<V>, world: &mut World)
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
    V::SubV: Componentable,
{
    ShapeEntityBuilderV::new_from_ref_shape(
        ref_shapes,
        ShapeLabel::from_str(INVERTED_CUBE_LABEL_STR),
    )
    .with_scale(crate::geometry::transform::Scaling::Scalar(4.0))
    .with_texturing_fn(color_cube_texture)
    .with_collider(Some(StaticCollider))
    .build(world)
    .build();

    ShapeEntityBuilderV::new_from_ref_shape(ref_shapes, ShapeLabel::from_str(CUBE_LABEL_STR))
        .with_translation(V::one_hot(0) * 6.0)
        .with_collider(Some(StaticCollider))
        .build(world)
        .build();
}
