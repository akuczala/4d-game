use crate::components::{Cursor,Transform};
use crate::draw::texture::color_cube_texture;
use crate::ecs_utils::Componentable;
use crate::geometry::transform::{Transformable, Scaling};
use crate::graphics::colors::*;
use crate::coin::Coin;
use specs::prelude::*;
use crate::shape_entity_builder::ShapeEntityBuilder;
use crate::vector::{Vec2,Vec3,Vec4};
use crate::geometry::shape::buildshapes::{build_duoprism_4d, ShapeBuilder, build_prism_2d, convex_shape_to_face_shape};
use crate::geometry::shape::{RefShapes, ShapeLabel};
use crate::constants::PI;
use crate::geometry::{Shape};
use crate::vector::{VectorTrait,Field};
use crate::draw::{self, ShapeTexture, FaceTexture, Texture};
use crate::collide::{StaticCollider};
use colored::Color::Magenta;

pub fn insert_wall<V, M>(world : &mut World, shape_builder : ShapeEntityBuilder<V, V::SubV, M>)
where
    V: VectorTrait<M = M> + Componentable,
    M: Componentable,
{
    shape_builder.build(world)
        .with(StaticCollider)
        .build();
}
pub fn insert_coin<V, M>(world : &mut World, shape_builder : ShapeEntityBuilder<V, V::SubV, M>)
where
    V: VectorTrait<M = M> + Componentable,
    M: Componentable,
{
    shape_builder.build(world)
        .with(Coin)
        .build();
}

fn build_test_walls<M, V>(build_shape: &ShapeEntityBuilder<V, V::SubV, M>, world: &mut World)
where
    V: VectorTrait<M = M> + Componentable,
    M: Componentable + Clone,
{
    let theta = PI/6.0;
    let cos = theta.cos();
    let sin = theta.sin();
    build_shape.clone()
        .with_translation(V::one_hot(-1)*(-1.0 - cos) + V::one_hot(1)*(sin - 1.0))
        .with_rotation(-1, 1, PI/2.0 - theta)
        .with_color(RED)
        .build(world)
        .with(StaticCollider)
        .build();
    build_shape.clone()
        .with_translation(V::one_hot(-1)*1.0)
        .with_rotation(0,-1,PI)
        .with_color(GREEN)
        .build(world)
        .with(StaticCollider)
        .build();
    build_shape.clone()
        .with_translation(V::one_hot(0)*1.0)
        .with_rotation(0,-1,PI/2.)
        .with_color(ORANGE)
        .build(world)
        .with(StaticCollider)
        .build();
    build_shape.clone()
        .with_translation(-V::one_hot(0)*1.0)
        .with_rotation(0,-1,3.0*PI/2.)
        .with_color(CYAN)
        .build(world)
        .with(StaticCollider)
        .build();
    let floor = build_shape.clone()
        .with_translation(-V::one_hot(1)*1.0)
        .with_rotation(-1,1,PI/2.)
        .with_color(BLUE);
    floor.clone().with_translation(-V::one_hot(0)*2.0).build(world).with(StaticCollider).build();
    floor.clone().with_translation(-V::one_hot(0)*2.0 - V::one_hot(-1)*2.0).build(world).with(StaticCollider).build();
    floor.clone()
        .with_translation(V::one_hot(1)*(2.0*sin) - V::one_hot(-1)*(2.0 + 2.0*cos))
        .build(world)
        .with(StaticCollider)
        .build();
    floor.clone()
        .with_translation(V::one_hot(1)*(2.0*sin) - V::one_hot(-1)*(4.0 + 2.0*cos))
        .with_color(MAGENTA)
        .build(world)
        .with(StaticCollider)
        .build();
    floor.build(world)
        .with(StaticCollider)
        .build();
    build_shape.clone()
        .with_translation(V::one_hot(1)*1.0)
        .with_rotation(-1,1,-PI/2.)
        .with_color(YELLOW)
        .build(world)
        .with(StaticCollider)
        .build();
}
pub fn build_test_level<V, M>(world: &mut World, ref_shapes: &mut RefShapes<V>)
    where
        V: VectorTrait<M = M> + Componentable,
        M: Componentable + Clone
{
    let (n_divisions, frame_vertis) = match V::DIM {
        3 => (vec![4,4], vec![1,3]),
        4 => (vec![4,4,4], vec![1,3,4]),
        _ => panic!("Cannot build test level in {} dimensions.", {V::DIM})
    };
    let sub_wall = ShapeBuilder::<V::SubV>::build_cube(2.0).build();
    let wall_label = ShapeLabel("Wall".to_string());
    let (wall, wall_single_face) = convex_shape_to_face_shape(sub_wall.clone(), true);
    ref_shapes.insert(wall_label.clone(), wall);
    let build_shape: ShapeEntityBuilder<V, V::SubV, V::M>= ShapeEntityBuilder::new_face_from_ref_shape(
        ref_shapes, wall_single_face, wall_label.clone())
        .with_face_texture(
            FaceTexture{
                texture: draw::Texture::make_tile_texture(&vec![0.8],&n_divisions),
                texture_mapping: Some(draw::TextureMapping{origin_verti : 0, frame_vertis})
            }
        );
    build_test_walls(&build_shape, world);
}

pub fn build_fun_level<V, M>(world: &mut World, ref_shapes: &mut RefShapes<V>)
    where 
    V: VectorTrait<M = M> + Componentable,
    M: Componentable + Clone
{
    let (n_divisions, frame_vertis) = match V::DIM {
        3 => (vec![4,4], vec![1,3]),
        4 => (vec![4,4,4], vec![1,3,4]),
        _ => panic!("Cannot build test level in {} dimensions.", {V::DIM})
    };
    let len = 4.0;
    let sub_cube = ShapeBuilder::<V::SubV>::build_cube(len).build();
    let wall_label = ShapeLabel("Wall".to_string());
    let (wall, wall_single_face) = convex_shape_to_face_shape(sub_cube.clone(), false);
    ref_shapes.insert(wall_label.clone(), wall);
    
    
    let wall_builder: ShapeEntityBuilder<V, V::SubV, V::M>= ShapeEntityBuilder::new_face_from_ref_shape(
        ref_shapes, wall_single_face, wall_label.clone()
    ).with_face_texture(
        FaceTexture {
            texture: draw::Texture::make_tile_texture(&vec![0.8],&n_divisions),
            texture_mapping: Some(draw::TextureMapping{origin_verti : 0, frame_vertis}),
        } 
    );
    let (floor, floor_single_face) = convex_shape_to_face_shape(sub_cube.clone(), true);
    let floor_label = ShapeLabel("Floor".to_string());
    ref_shapes.insert(floor_label.clone(), floor);
    let upper_floor_builder = ShapeEntityBuilder::new_face_from_ref_shape(
       &ref_shapes, floor_single_face, floor_label.clone()
    ).stretch(&(V::ones()*0.5-V::one_hot(1)*0.25)).with_rotation(-1, 1, -PI/2.0)
        .with_translation(V::one_hot(1)*len/2.0);
    // upper_floor_builder.build(world)
    //     .with(StaticCollider).build();
    let colors = vec![RED,GREEN,BLUE,CYAN,MAGENTA,YELLOW,ORANGE,WHITE];
    // floors are all misaligned and the wrong size. did this ever work?
    for ((i,sign),color) in iproduct!(0..V::DIM, vec![-1,1]).zip(colors.into_iter()) {
        if !(i == 1 && sign == 1) {
            let float_sign = sign as Field;
            let shape = wall_builder.clone()
                .with_translation(V::one_hot(i) * float_sign * len/2.0);
            match i == V::DIM-1 {
                true => shape.with_rotation(1, i, -PI * (1.0 + float_sign) / 2.0),
                false => shape.with_rotation(-1, i, -PI / 2.0 * float_sign)
            }.with_color(color)
                .build(world)
                .with(StaticCollider).build();
        }
        if i != 1 {
            let float_sign = sign as Field;
            let shape = upper_floor_builder.clone()
                .with_translation(V::one_hot(i) * float_sign * len*(1.5));
            match i == V::DIM-1 {
                true => shape,
                false => shape.with_rotation(-1, i, -PI / 2.0 * float_sign)
            }.with_color(color)
                .build(world)
                .with(StaticCollider).build();
        }
    }

}

pub fn build_scene<V, U, M>(world : &mut World)
where
    V: VectorTrait<M = M, SubV =U> + Componentable,
    U: VectorTrait + Componentable,
    M: Componentable + Clone
{
    let mut ref_shapes: RefShapes<V> = RefShapes::new();
    build_lvl_1(world, &mut ref_shapes);
    //build_fun_level::<V>(world, &mut ref_shapes);
    //build_test_level::<V>(world, &mut ref_shapes);
    //build_test_face(world);
    init_player(world, V::zero());
    world.insert(ref_shapes);
}

pub fn build_corridor_cross<V : VectorTrait>(cube_builder: &ShapeEntityBuilder<V, V::SubV, V::M>, wall_length : Field) -> Vec<ShapeEntityBuilder<V, V::SubV, V::M>> {

    // todo figure out why texture is now off after changes to transform
    pub fn build_texture<V : VectorTrait>(shape : &Shape<V>, scale: &Scaling<V>) -> ShapeTexture<V::SubV> {
        let mut cube_texture = color_cube_texture(shape);
        for (face, face_texture) in shape.faces.iter().zip(cube_texture.face_textures.iter_mut()) {
            let target_face_color = match face_texture.texture {
            draw::Texture::DefaultLines{color} => color,
            _ => panic!("build corridor cross expected DefaultLines") //don't bother handling the other cases
            };
            //let face_scales = linspace(0.1,0.9,5).collect();
            let face_scales = vec![0.95];
            face_texture.texture = draw::Texture::make_tile_texture(&face_scales,
            & match V::DIM {
                3 => vec![3, 1],
                4 => vec![3, 1, 1],
                _ => panic!()
            })
            //.merged_with(&draw::Texture::make_fuzz_texture(1000).set_color(target_face_color))
            .set_color(target_face_color);

            // must use scaled verts to properly align textures
            let scaled_verts = shape.verts.iter().map(|v| scale.scale_vec(*v)).collect();
            face_texture.texture_mapping = Some(draw::TextureMapping::calc_cube_vertis(face,&scaled_verts,&shape.edges))
        }
        cube_texture
    }
    let corr_width = 1.0;
    let wall_height = 1.0;
    //let origin = V::zero();
    let signs = vec![-1.0,1.0];
    let axis_pairs = match V::DIM {
        3 => vec![(0,2)],
        4 => vec![(0,2),(2,3),(3,0)],
        _ => panic!("Invalid dimension for build_corridor_cross")
    };
    let axes = match V::DIM {
        3 => -1..1,
        4 => -2..1,
        _ => panic!("Invalid dimension for build_corridor_cross")
    };
    
    let mut shape_builders : Vec<ShapeEntityBuilder<V, V::SubV, V::M>> = Vec::new();
    //corridor walls
    let mut walls1 : Vec<ShapeEntityBuilder<V, V::SubV, V::M>> = iproduct!(signs.iter(),signs.iter(),axis_pairs.iter())
        .map(|(s1,s2,(ax1,ax2))|
            cube_builder.clone()
            .with_translation(
                V::one_hot(*ax1)*(*s1)*(corr_width+wall_length)/2.0
                + V::one_hot(*ax2)*(*s2)*(corr_width+wall_length)/2.0
                )
            .stretch(&(V::one_hot(1)*(wall_height - corr_width)
                + V::one_hot(*ax1)*(wall_length - corr_width)
                + V::one_hot(*ax2)*(wall_length - corr_width)
                + V::ones()*corr_width
            ))
            ).collect();
    for builder in &mut walls1 {
        builder.shape_texture = build_texture(&mut builder.shape, &builder.transformation.scale);
    }

    shape_builders.append(&mut walls1);

    //end walls
    
    let mut end_walls = iproduct!(axes.clone(),signs.iter())
        .map(|(i,sign)|
            cube_builder.clone()
                .with_translation(V::one_hot(i)*(wall_length+corr_width)*(*sign))
                .stretch(&(
                    V::one_hot(1)*(wall_height-corr_width) + V::ones()*corr_width
                    ))
                .with_texturing_fn(color_cube_texture)
                );
    shape_builders.append(&mut end_walls.collect());
    //floors and ceilings
    let mut floors_long : Vec<ShapeEntityBuilder<V, V::SubV, V::M>> = iproduct!(axes.clone(),signs.iter())
        .map(|(i,sign)|
            cube_builder.clone()
                .with_translation(V::one_hot(i)*(wall_length+corr_width)*(*sign)/2.0
                    - V::one_hot(1)*(wall_height + corr_width)/2.0
                    )
                .stretch(&(V::one_hot(i)*(wall_length-corr_width) + V::ones()*corr_width
                    ))
                ).collect();
    let mut ceilings_long : Vec<ShapeEntityBuilder<V, V::SubV, V::M>> = floors_long.iter()
        .map(|block| block.clone().with_translation(
            V::one_hot(1)*(wall_height+corr_width)
        ))
        .collect();

    for builder in &mut floors_long {
        builder.shape_texture = build_texture(&mut builder.shape, &builder.transformation.scale);
    }
    for builder in &mut ceilings_long {
        builder.shape_texture = build_texture(&mut builder.shape, &builder.transformation.scale);
    }

    shape_builders.append(&mut floors_long);
    shape_builders.append(&mut ceilings_long);
    //center floor
    shape_builders.push(
        cube_builder.clone()
            .with_translation(-V::one_hot(1)*(wall_height + corr_width)/2.0)
            .with_texturing_fn(color_cube_texture)
    );
    shape_builders
    
}
pub fn init_player<V, U, M>(world: &mut World, pos: V)
    where
    V: VectorTrait<M = M, SubV =U> + Componentable,
    U: VectorTrait + Componentable,
    M: Componentable + Clone
{
    let transform = Transform::identity().with_translation(pos);
    crate::player::build_player(world, &transform);
    init_cursor::<V, V::SubV>(world);

}
pub fn init_cursor<V, U>(world: &mut World)
where
    V: VectorTrait<SubV= U> + Componentable,
    U: VectorTrait + Componentable
{
    world.create_entity()
        .with(Cursor)
        .with(ShapeBuilder::<V::SubV>::build_cube(0.03).build())
        .build();
}

pub fn build_lvl_1<V, M>(world : &mut World, ref_shapes: &mut RefShapes<V>)
    where
    V: VectorTrait<M = M> + Componentable,
    M: Componentable + Clone
{
    let cube = ShapeBuilder::<V>::build_cube(1.0).build();
    let coin: Shape<V> = ShapeBuilder::<V>::build_coin().build();
    let cube_label = ShapeLabel("Cube".to_string());
    let coin_label = ShapeLabel("Coin".to_string());
    ref_shapes.insert(cube_label.clone(), cube);
    ref_shapes.insert(coin_label.clone(), coin);
    let cube_builder = ShapeEntityBuilder::new_convex_from_ref_shape(ref_shapes, cube_label.clone());

    let wall_length = 3.0;
    let walls : Vec<ShapeEntityBuilder<V, V::M>> = build_corridor_cross(&cube_builder, wall_length);

    for wall in walls.into_iter() {
        insert_wall(world,wall)
    }
    //let (m,n) = (4,4);
    //let mut duocylinder = buildshapes::build_duoprism_4d([1.0,1.0],[[0,1],[2,3]],[m,n])
    for (axis,dir) in iproduct!(match V::DIM {3 => vec![0,2], 4 => vec![0,2,3], _ => panic!("Invalid dimension")},vec![-1.,1.]) {
        insert_coin(
            world,
            ShapeEntityBuilder::new_convex_from_ref_shape(&ref_shapes, coin_label.clone())
                .with_translation(V::one_hot(axis)*dir*(wall_length - 0.5))
                .with_color(YELLOW)
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