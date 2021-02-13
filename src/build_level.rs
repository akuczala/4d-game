use crate::components::Cursor;

use crate::clipping::ShapeClipState;
use crate::camera::Camera;
use crate::coin::Coin;
use specs::prelude::*;
use crate::vector::{Vec2,Vec3,Vec4,linspace};
use crate::geometry::buildshapes::{build_cube_4d,color_cube,build_duoprism_4d,ShapeBuilder};

use crate::geometry::{Shape,buildshapes,shape::{ShapeType,Convex}};
use crate::vector::{VectorTrait,Field};
use crate::draw;
use crate::collide::{StaticCollider,HasBBox};

pub fn insert_wall<V : VectorTrait>(world : &mut World, shape : Shape<V>) {
    world.create_entity()
        .with(shape.calc_bbox())
        .with(ShapeType::Convex(Convex::new(&shape)))
        .with(shape)
        .with(ShapeClipState::<V>::default())
        .with(StaticCollider)
        .build();
}
pub fn insert_coin<V : VectorTrait>(world : &mut World, shape : Shape<V>) {
    world.create_entity()
        .with(shape.calc_bbox())
        .with(ShapeType::Convex(Convex::new(&shape)))
        .with(shape)
        .with(ShapeClipState::<V>::default())
        .with(Coin)
        .build();
}
pub fn build_shapes_3d(world : &mut World) {

    build_lvl_1_3d(world);

    world.create_entity()
        .with(Cursor)
        .with(ShapeBuilder::<Vec2>::build_cube(0.03))
        .build();
}
pub fn build_shapes_4d(world : &mut World) {
    build_lvl_1_4d(world);
    world.create_entity()
        .with(Cursor)
        .with(ShapeBuilder::<Vec3>::build_cube(0.03))
        .build();
    
}

pub fn build_corridor_cross<V : VectorTrait>(cube : &Shape<V>, wall_length : Field) -> Vec<Shape<V>> {

    pub fn apply_texture<V : VectorTrait>(shape : &mut Shape<V>) {
        for face in shape.faces.iter_mut() {
            let target_face_color = match face.texture {
            draw::Texture::DefaultLines{color} => color,
            _ => panic!("build corridor cross expected DefaultLines") //don't bother handling the other cases
            };
            //let face_scales = linspace(0.1,0.9,5).collect();
            let face_scales = vec![0.9];
            face.texture = draw::Texture::make_tile_texture(&face_scales,
            & match V::DIM {
                3 => vec![3,1],
                4 => vec![3,1,1],
                _ => panic!()
            }).set_color(target_face_color);
            face.texture_mapping = draw::TextureMapping::calc_cube_vertis(face,&shape.verts,&shape.edges)
        }
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
        3 => (-1..1),
        4 => (-2..1),
        _ => panic!("Invalid dimension for build_corridor_cross")
    };
    
    let mut shapes : Vec<Shape<V>> = Vec::new();
    //corridor walls
    let mut walls1 : Vec<Shape<V>> = iproduct!(signs.iter(),signs.iter(),axis_pairs.iter())
        .map(|(s1,s2,(ax1,ax2))| cube.clone()
            .set_pos(&(
                V::one_hot(*ax1)*(*s1)*(corr_width+wall_length)/2.0
                + V::one_hot(*ax2)*(*s2)*(corr_width+wall_length)/2.0
                ))
            .stretch(&(V::one_hot(1)*(wall_height - corr_width)
                + V::one_hot(*ax1)*(wall_length - corr_width)
                + V::one_hot(*ax2)*(wall_length - corr_width)
                + V::ones()*corr_width
                    ))
            ).collect();
    for shape in &mut walls1 {
        apply_texture(shape);
    }
    //test texturing
    // for shape in &mut walls1 {
    //         let face = &mut shape.faces[0];
    //         //print!("{}",face);
    //         let target_face_color = match face.texture {
    //             draw::Texture::DefaultLines{color} => color,
    //             _ => panic!("build corridor cross expected DefaultLines") //don't bother handling the other cases
    //         };
    //         face.texture = draw::Texture::make_tile_texture(&vec![0.9],
    //         & match V::DIM {
    //             3 => vec![3,1],
    //             4 => vec![3,1,1],
    //             _ => panic!()
    //         }).set_color(target_face_color);
    //         face.texture_mapping = draw::TextureMapping{origin_verti : 0,
    //         frame_vertis : match V::DIM {
    //             3 => vec![face.vertis[3],face.vertis[1]],
    //             4 => vec![face.vertis[3],face.vertis[1],face.vertis[4]], _ => panic!()}
    //         };
    // }
    

    shapes.append(&mut walls1);

    //end walls
    
    let walls2 = iproduct!(axes.clone(),signs.iter())
        .map(|(i,sign)| cube.clone()
                .set_pos(&(V::one_hot(i)*(wall_length+corr_width)*(*sign)))
                .stretch(&(
                    V::one_hot(1)*(wall_height-corr_width) + V::ones()*corr_width
                    ))
                );
    shapes.append(&mut walls2.collect());
    //floors and ceilings
    let mut floors_long : Vec<Shape<V>> = iproduct!(axes.clone(),signs.iter())
        .map(|(i,sign)| cube.clone()
                .set_pos(&(V::one_hot(i)*(wall_length+corr_width)*(*sign)/2.0
                    - V::one_hot(1)*(wall_height + corr_width)/2.0
                    ))
                .stretch(&(V::one_hot(i)*(wall_length-corr_width) + V::ones()*corr_width
                    ))
                ).collect();
    let mut ceilings_long : Vec<Shape<V>> = floors_long.iter()
        .map(|block| block.clone().set_pos(&(
            *block.get_pos()+V::one_hot(1)*(wall_height+corr_width))
        ))
        .collect();

    for shape in &mut floors_long {
        apply_texture(shape);
    }
    for shape in &mut ceilings_long {
        apply_texture(shape);
    }

    shapes.append(&mut floors_long);
    shapes.append(&mut ceilings_long);
    //center floor
    shapes.push(cube.clone().set_pos(&(-V::one_hot(1)*(wall_height + corr_width)/2.0)));
    shapes
    
}

pub fn build_lvl_1_3d(world : &mut World) {
    build_lvl_1(world,ShapeBuilder::<Vec3>::build_cube(1.0),ShapeBuilder::<Vec3>::build_coin());
}
pub fn build_lvl_1_4d(world : &mut World) {
    build_lvl_1(world,ShapeBuilder::<Vec4>::build_cube(1.0),ShapeBuilder::<Vec4>::build_coin());
}

pub fn build_lvl_1<V : VectorTrait>(world : &mut World, cube : Shape<V>, coin : Shape<V>) {
    let wall_length = 3.0;
    //buildshapes::build_axes_cubes_4d()
    //buildshapes::cubeidor_4d()
    let walls : Vec<Shape<V>> = build_corridor_cross(
        &color_cube(cube),wall_length);

    for wall in walls.into_iter() {
        insert_wall(world,wall)
    }
    //let (m,n) = (4,4);
    //let mut duocylinder = buildshapes::build_duoprism_4d([1.0,1.0],[[0,1],[2,3]],[m,n])
    for (axis,dir) in iproduct!(match V::DIM {3 => vec![0,2], 4 => vec![0,2,3], _ => panic!("Invalid dimension")},vec![-1.,1.]) {
        insert_coin(world,
            coin.clone()
                .set_pos(&(V::one_hot(axis)*dir*(wall_length - 0.5)))
        );
    }
    let camera = Camera::new(V::zero());
    crate::player::build_player(world, camera);

}

pub fn build_test_scene_3d() -> Vec<Shape<Vec3>> {
    let mut cube = buildshapes::build_cube_3d(1.0);
    //let cube_2 = cube.clone().set_pos(&Vec3::new(0.0,0.0,3.0)).stretch(&Vec3::new(1.0,8.0,1.0));
    let cube_3 = cube.clone().set_pos(&Vec3::new(-2.0,0.0,0.0)).stretch(&Vec3::new(2.0,2.0,2.0));

    //test texture'
    cube.faces[0].texture = draw::Texture::make_tile_texture(&vec![0.5,0.9],&vec![2,3]);
    cube.faces[0].texture_mapping = draw::TextureMapping{origin_verti : 0, frame_vertis : vec![1,3]};

    let shapes = vec![cube,cube_3];
    for shape in &shapes {
        println!("radius:{}", shape.radius);
    }
    shapes
}