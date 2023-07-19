use specs::{World, WorldExt};

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

fn build_corridor_cross<V: VectorTrait>(
    cube_builder: &ShapeEntityBuilderV<V>,
    wall_length: Field,
    config: &Config,
    open_center: bool,
) -> Vec<ShapeEntityBuilderV<V>> {
    // todo figure out why texture is now off after changes to transform
    pub fn build_texture<V: VectorTrait>(
        shape: &Shape<V>,
        scale: &Scaling<V>,
        config: &Config,
    ) -> ShapeTexture<V::SubV> {
        let mut cube_texture = color_cube_texture(shape);
        for (face, face_texture) in shape
            .faces
            .iter()
            .zip(cube_texture.face_textures.iter_mut())
        {
            let target_face_color = match face_texture.texture {
                draw::Texture::DefaultLines { color } => color,
                _ => panic!("build corridor cross expected DefaultLines"), //don't bother handling the other cases
            };
            face_texture.texture = draw::Texture::make_tile_texture(
                &[FACE_SCALE],
                &match V::DIM {
                    3 => vec![3, 1],
                    4 => vec![3, 1, 1],
                    _ => panic!(),
                },
            )
            .merged_with(
                &draw::Texture::make_fuzz_texture(config.fuzz_lines.face_num)
                    .set_color(target_face_color),
            )
            .set_color(target_face_color);

            // must use scaled verts to properly align textures
            let scaled_verts: Vec<V> = shape.verts.iter().map(|v| scale.scale_vec(*v)).collect();
            face_texture.texture_mapping = Some(draw::TextureMapping::calc_cube_vertis(
                face,
                &scaled_verts,
                &shape.edges,
            ))
        }
        cube_texture
    }
    let corr_width = 1.0;
    let wall_height = 1.0;
    //let origin = V::zero();
    let signs = vec![-1.0, 1.0];
    let axis_pairs = match V::DIM {
        3 => vec![(0, 2)],
        4 => vec![(0, 2), (2, 3), (3, 0)],
        _ => panic!("Invalid dimension for build_corridor_cross"),
    };
    let axes = match V::DIM {
        3 => -1..1,
        4 => -2..1,
        _ => panic!("Invalid dimension for build_corridor_cross"),
    };

    let mut shape_builders: Vec<ShapeEntityBuilderV<V>> = Vec::new();
    //corridor walls
    let mut walls1: Vec<ShapeEntityBuilderV<V>> =
        iproduct!(signs.iter(), signs.iter(), axis_pairs.iter())
            .map(|(s1, s2, (ax1, ax2))| {
                cube_builder
                    .clone()
                    .with_translation(
                        V::one_hot(*ax1) * (*s1) * (corr_width + wall_length) / 2.0
                            + V::one_hot(*ax2) * (*s2) * (corr_width + wall_length) / 2.0,
                    )
                    .stretch(
                        &(V::one_hot(1) * (wall_height - corr_width)
                            + V::one_hot(*ax1) * (wall_length - corr_width)
                            + V::one_hot(*ax2) * (wall_length - corr_width)
                            + V::ones() * corr_width),
                    )
            })
            .collect();
    for builder in &mut walls1 {
        builder.shape_texture =
            build_texture(&builder.shape, &builder.transformation.scale, config);
    }

    shape_builders.append(&mut walls1);

    //end walls

    let end_walls = iproduct!(axes.clone(), signs.iter()).map(|(i, sign)| {
        cube_builder
            .clone()
            .with_translation(V::one_hot(i) * (wall_length + corr_width) * (*sign))
            .stretch(&(V::one_hot(1) * (wall_height - corr_width) + V::ones() * corr_width))
            .with_texturing_fn(|s| fuzzy_color_cube_texture(s, config.fuzz_lines.face_num))
    });
    shape_builders.append(&mut end_walls.collect());
    //floors and ceilings
    let mut floors_long: Vec<ShapeEntityBuilderV<V>> = iproduct!(axes, signs.iter())
        .map(|(i, sign)| {
            cube_builder
                .clone()
                .with_translation(
                    V::one_hot(i) * (wall_length + corr_width) * (*sign) / 2.0
                        - V::one_hot(1) * (wall_height + corr_width) / 2.0,
                )
                .stretch(&(V::one_hot(i) * (wall_length - corr_width) + V::ones() * corr_width))
        })
        .collect();
    let mut ceilings_long: Vec<ShapeEntityBuilderV<V>> = floors_long
        .iter()
        .map(|block| {
            block
                .clone()
                .with_translation(V::one_hot(1) * (wall_height + corr_width))
        })
        .collect();

    for builder in &mut floors_long {
        builder.shape_texture =
            build_texture(&builder.shape, &builder.transformation.scale, config);
    }
    for builder in &mut ceilings_long {
        builder.shape_texture =
            build_texture(&builder.shape, &builder.transformation.scale, config);
    }

    shape_builders.append(&mut floors_long);
    shape_builders.append(&mut ceilings_long);
    //center floor
    shape_builders.push(
        cube_builder
            .clone()
            .with_translation(-V::one_hot(1) * (wall_height + corr_width) / 2.0)
            .with_texturing_fn(|shape| fuzzy_color_cube_texture(shape, config.fuzz_lines.face_num)),
    );
    //center ceiling
    if !open_center {
        shape_builders.push(
            shape_builders[shape_builders.len() - 1]
                .clone()
                .with_translation(V::one_hot(1) * (wall_height + corr_width)),
        );
    }

    shape_builders
}

pub fn build_lvl_1<V>(world: &mut World, ref_shapes: &RefShapes<V>, open_center: bool)
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    let cube_builder =
        ShapeEntityBuilder::new_from_ref_shape(ref_shapes, ShapeLabel::from_str(CUBE_LABEL_STR));

    let wall_length = 3.0;
    let config = (*world.read_resource::<Config>()).clone();
    let walls: Vec<ShapeEntityBuilderV<V>> =
        build_corridor_cross(&cube_builder, wall_length, &config, open_center);

    for wall in walls.into_iter() {
        insert_static_collider(world, wall)
    }
    //let (m,n) = (4,4);
    //let mut duocylinder = buildshapes::build_duoprism_4d([1.0,1.0],[[0,1],[2,3]],[m,n])
    for (axis, dir) in iproduct!(
        match V::DIM {
            3 => vec![0, 2],
            4 => vec![0, 2, 3],
            _ => panic!("Invalid dimension"),
        },
        vec![-1., 1.]
    ) {
        insert_coin(
            world,
            ShapeEntityBuilder::new_from_ref_shape(
                ref_shapes,
                ShapeLabel::from_str(COIN_LABEL_STR),
            )
            .with_translation(V::one_hot(axis) * dir * (wall_length - 0.5))
            .with_color(YELLOW),
        );
    }
}
// pub fn build_lvl_1_with_faces<V : VectorTrait>(world : &mut World, ref_shapes: &mut RefShapes<V>) {
//     let square_builder = ShapeBuilder::<V::SubV>::build_cube(1.0);
//     let wall_length = 3.0;
//     let rect_builder = square_builder.clone().stretch(&(V::one_hot(0)*wall_length));
//     // let tube =
//     let coin = ShapeBuilder::<V>::build_coin().build();
//
//
//
// }
