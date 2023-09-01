use crate::constants::{CARDINAL_COLORS, ONE_SIDED_FACE_LABEL_STR, PI, TWO_SIDED_FACE_LABEL_STR};

use crate::draw::texture::texture_builder::TextureBuilder;

use crate::utils::ValidDimension;
use crate::{
    components::{ShapeLabel, Transformable},
    geometry::transform::Scaling,
    shape_entity_builder::{ShapeEntityBuilder, ShapeEntityBuilderV},
    vector::{Field, VectorTrait},
};

pub fn build_fun_level<V: VectorTrait>() -> Vec<ShapeEntityBuilderV<V>> {
    let n_divisions = match V::DIM.into() {
        ValidDimension::Three => vec![2, 2],
        ValidDimension::Four => vec![2, 2, 2],
    };
    let len = 4.0;
    let wall_label = ShapeLabel::from(ONE_SIDED_FACE_LABEL_STR);
    let wall_builder = ShapeEntityBuilder::new(wall_label)
        .with_scale(Scaling::Scalar(len))
        .with_face_texture(
            TextureBuilder::new_tile_texture(vec![0.8], n_divisions)
                .merged_with(TextureBuilder::new_fuzz_texture()),
        );
    let floor_label = ShapeLabel::from(TWO_SIDED_FACE_LABEL_STR);
    let upper_floor_builder = ShapeEntityBuilder::new(floor_label)
        .with_scale(Scaling::Scalar(len))
        .stretch(&(V::ones() * 0.5 - V::one_hot(1) * 0.25))
        .with_rotation(-1, 1, -PI / 2.0)
        .with_translation(V::one_hot(1) * len / 2.0);
    // upper_floor_builder.build(world)
    //     .with(StaticCollider).build();
    let colors = CARDINAL_COLORS;
    let mut builders = Vec::new();
    // floors are all misaligned and the wrong size. did this ever work?
    for ((i, sign), color) in iproduct!(0..V::DIM, vec![-1, 1]).zip(colors.into_iter()) {
        if !(i == 1 && sign == 1) {
            let float_sign = sign as Field;
            let mut shape = wall_builder
                .clone()
                .with_translation(V::one_hot(i) * float_sign * len / 2.0);
            shape = match i == V::DIM - 1 {
                true => shape.with_rotation(1, i, -PI * (1.0 + float_sign) / 2.0),
                false => shape.with_rotation(-1, i, -PI / 2.0 * float_sign),
            }
            .with_color(color);
            builders.push(shape);
        }
        if i != 1 {
            let float_sign = sign as Field;
            let mut shape = upper_floor_builder
                .clone()
                .with_translation(V::one_hot(i) * float_sign * len * (1.5));
            shape = match i == V::DIM - 1 {
                true => shape,
                false => shape.with_rotation(-1, i, -PI / 2.0 * float_sign),
            }
            .with_color(color);
            builders.push(shape);
        }
    }
    builders
}
