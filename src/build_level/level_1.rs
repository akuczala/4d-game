use specs::World;

use crate::{
    components::{RefShapes, Transformable},
    constants::{COIN_LABEL_STR, CUBE_LABEL_STR, FUZZY_TILE_LABEL_STR},
    draw::texture::ShapeTextureBuilder,
    ecs_utils::Componentable,
    graphics::colors::YELLOW,
    shape_entity_builder::{ShapeEntityBuilder, ShapeEntityBuilderV},
    utils::ValidDimension,
    vector::{Field, VectorTrait},
};

use super::{insert_coin, insert_static_collider};

fn build_corridor_cross<V: VectorTrait>(
    cube_builder: &ShapeEntityBuilderV<V>,
    wall_length: Field,
    open_center: bool,
) -> Vec<ShapeEntityBuilderV<V>> {
    let corr_width = 1.0;
    let wall_height = 1.0;
    //let origin = V::zero();
    let signs = vec![-1.0, 1.0];
    let axis_pairs = match V::DIM.into() {
        ValidDimension::Three => vec![(0, 2)],
        ValidDimension::Four => vec![(0, 2), (2, 3), (3, 0)],
    };
    let axes = match V::DIM.into() {
        ValidDimension::Three => -1..1,
        ValidDimension::Four => -2..1,
    };

    let mut shape_builders: Vec<ShapeEntityBuilderV<V>> = Vec::new();
    let fuzzy_tile_tex = ShapeTextureBuilder::from_resource(FUZZY_TILE_LABEL_STR.into());
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
        take_mut::take(builder, |b| b.with_texture(fuzzy_tile_tex.clone()));
    }

    shape_builders.append(&mut walls1);

    //end walls

    let end_walls = iproduct!(axes.clone(), signs.iter()).map(|(i, sign)| {
        cube_builder
            .clone()
            .with_translation(V::one_hot(i) * (wall_length + corr_width) * (*sign))
            .stretch(&(V::one_hot(1) * (wall_height - corr_width) + V::ones() * corr_width))
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
        take_mut::take(builder, |b| b.with_texture(fuzzy_tile_tex.clone()));
    }
    for builder in &mut ceilings_long {
        take_mut::take(builder, |b| b.with_texture(fuzzy_tile_tex.clone()));
    }

    shape_builders.append(&mut floors_long);
    shape_builders.append(&mut ceilings_long);
    //center floor
    shape_builders.push(
        cube_builder
            .clone()
            .with_translation(-V::one_hot(1) * (wall_height + corr_width) / 2.0),
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
    let cube_builder = ShapeEntityBuilder::new(CUBE_LABEL_STR.into());

    let wall_length = 3.0;
    let walls: Vec<ShapeEntityBuilderV<V>> =
        build_corridor_cross(&cube_builder, wall_length, open_center);

    for wall in walls.into_iter() {
        insert_static_collider(world, ref_shapes, wall)
    }
    //let (m,n) = (4,4);
    //let mut duocylinder = buildshapes::build_duoprism_4d([1.0,1.0],[[0,1],[2,3]],[m,n])
    for (axis, dir) in iproduct!(
        match V::DIM.into() {
            ValidDimension::Three => vec![0, 2],
            ValidDimension::Four => vec![0, 2, 3],
        },
        vec![-1., 1.]
    ) {
        insert_coin(
            world,
            ref_shapes,
            ShapeEntityBuilder::new(COIN_LABEL_STR.into())
                .with_translation(V::one_hot(axis) * dir * (wall_length - 0.5))
                .with_color(YELLOW),
        );
    }
}
