use specs::{World, WorldExt};

use crate::constants::{CARDINAL_COLORS, PI, ONE_SIDED_FACE_LABEL_STR, TWO_SIDED_FACE_LABEL_STR};
use crate::graphics::colors::*;
use crate::{
    components::{RefShapes, Shape, ShapeLabel, ShapeTexture, Transformable},
    config::{Config, FuzzLinesConfig},
    draw::{
        self,
        texture::{color_cube_texture, fuzzy_color_cube_texture},
        FaceTexture, Texture,
    },
    ecs_utils::Componentable,
    geometry::{
        shape::buildshapes::{convex_shape_to_face_shape, ShapeBuilder},
        transform::Scaling,
    },
    graphics::colors::YELLOW,
    shape_entity_builder::{ShapeEntityBuilder, ShapeEntityBuilderV},
    vector::{Field, VectorTrait},
};

use super::{insert_coin, insert_static_collider};

pub fn build_fun_level<V: VectorTrait>(
    fuzz_config: FuzzLinesConfig,
    ref_shapes: &RefShapes<V>,
) -> Vec<ShapeEntityBuilderV<V>> {
    let (n_divisions, frame_vertis) = match V::DIM {
        3 => (vec![2, 2], vec![1, 3]),
        4 => (vec![2, 2, 2], vec![1, 3, 4]),
        _ => panic!("Cannot build test level in {} dimensions.", { V::DIM }),
    };
    let len = 4.0;
    let wall_label = ShapeLabel::from_str(ONE_SIDED_FACE_LABEL_STR);

    let wall_builder = ShapeEntityBuilder::new_from_ref_shape(ref_shapes, wall_label)
    .with_scale(Scaling::Scalar(len))
        .with_face_texture(FaceTexture {
            texture: draw::Texture::make_tile_texture(&[0.8], &n_divisions)
                .merged_with(&Texture::make_fuzz_texture(fuzz_config.face_num)),
            texture_mapping: Some(draw::TextureMapping {
                origin_verti: 0,
                frame_vertis,
            }),
        });
    let floor_label = ShapeLabel::from_str(TWO_SIDED_FACE_LABEL_STR);
    let upper_floor_builder = ShapeEntityBuilder::new_from_ref_shape(ref_shapes, floor_label)
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
