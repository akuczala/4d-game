use crate::components::{Cursor,Transform,BBox};
use crate::geometry::transform::{Transformable};
use crate::colors::*;
use crate::clipping::ShapeClipState;
use crate::camera::Camera;
use crate::coin::Coin;
use specs::prelude::*;
use crate::vector::{Vec2,Vec3,Vec4,linspace};
use crate::geometry::shape::buildshapes;
use crate::geometry::shape::buildshapes::{build_cube_3d, build_cube_4d, color_cube, build_duoprism_4d, ShapeBuilder, build_prism_2d};
use crate::constants::PI;
use crate::geometry::{Shape, shape::{ShapeType, convex::Convex, single_face::SingleFace}, Face};
use crate::vector::{VectorTrait,Field};
use crate::draw;
use crate::collide::{StaticCollider,HasBBox};

pub fn insert_wall<V : VectorTrait>(world : &mut World, shape : Shape<V>) {
    world.create_entity()
        .with(shape.calc_bbox())
        .with(ShapeType::<V>::Convex(Convex::new(&shape)))
        .with(shape)
        .with(ShapeClipState::<V>::default())
        .with(StaticCollider)
        .build();
}
pub fn insert_coin<V : VectorTrait>(world : &mut World, shape : Shape<V>) {
    world.create_entity()
        .with(shape.calc_bbox())
        .with(ShapeType::<V>::Convex(Convex::new(&shape)))
        .with(shape)
        .with(ShapeClipState::<V>::default())
        .with(Coin)
        .build();
}
#[derive(Clone)]
struct BuildFaceShape<V: VectorTrait> {
    sub_shape: Shape<V::SubV>,
    transformation: Transform<V>,
    texture_info: (draw::Texture<V::SubV>, draw::TextureMapping),
}
impl<V: VectorTrait> BuildFaceShape<V> {
    pub fn new(sub_shape: Shape<V::SubV>) -> Self {
        Self{
            sub_shape,
            transformation: Transform::identity(),
            texture_info: Default::default(),
        }
    }
    pub fn with_texture(mut self, texture: draw::Texture<V::SubV>, texture_mapping: draw::TextureMapping) -> Self {
        self.texture_info = (texture, texture_mapping);
        self
    }
    pub fn with_color(mut self, color: Color) -> Self {
        self.texture_info.0 = self.texture_info.0.set_color(color);
        self
    }
    pub fn build(self, world: &mut World) {
        let Self{sub_shape, transformation, texture_info} = self;
        let (mut shape, mut single_face) = buildshapes::convex_shape_to_face_shape(sub_shape);
        shape = shape.transform(transformation);
        shape.faces[0].set_texture(texture_info.0, texture_info.1);
        single_face.update(&shape);
        world.create_entity()
            .with(shape.calc_bbox())
            .with(ShapeType::SingleFace(single_face))
            .with(shape)
            .with(ShapeClipState::<V>::default())
            .with(StaticCollider)
            .build();
    }
}
impl<V: VectorTrait> Transformable<V> for BuildFaceShape<V> {
    fn set_identity(mut self) -> Self {
        self.transformation = Transform::identity();
        self
    }
    fn transform(mut self, transformation: Transform<V>) -> Self {
        self.transformation = self.transformation.transform(transformation);
        self
    }
}

fn build_test_walls<V: VectorTrait>(build_shape: &BuildFaceShape<V>, world: &mut World) {
    let theta = (PI/6.0);
    let cos = theta.cos();
    let sin = theta.sin();
    build_shape.clone()
        .with_translation(V::one_hot(-1)*(-1.0 - cos) + V::one_hot(1)*(sin - 1.0))
        .with_rotation(-1, 1, PI/2.0 - theta)
        .with_color(RED)
        .build(world);
    build_shape.clone()
        .with_translation(V::one_hot(-1)*1.0)
        .with_rotation(0,-1,PI)
        .with_color(GREEN)
        .build(world);
    build_shape.clone()
        .with_translation(V::one_hot(0)*1.0)
        .with_rotation(0,-1,PI/2.)
        .with_color(ORANGE)
        .build(world);
    build_shape.clone()
        .with_translation(-V::one_hot(0)*1.0)
        .with_rotation(0,-1,3.0*PI/2.)
        .with_color(CYAN)
        .build(world);
    let floor = build_shape.clone()
        .with_translation(-V::one_hot(1)*1.0)
        .with_rotation(-1,1,PI/2.)
        .with_color(BLUE);
    floor.clone()
        .with_translation(V::one_hot(1)*(2.0*sin) - V::one_hot(-1)*(2.0 + 2.0*cos))
        .build(world);
    floor.build(world);
    build_shape.clone()
        .with_translation(V::one_hot(1)*1.0)
        .with_rotation(-1,1,-PI/2.)
        .with_color(YELLOW)
        .build(world);
}
pub fn build_test_level_3d(world: &mut World) {
    insert_wall(world,build_cube_3d(1.0).with_pos(&Vec3::new(3., 0., 0.)));
    let build_shape: BuildFaceShape<Vec3>= BuildFaceShape::new(ShapeBuilder::<Vec2>::build_cube(2.0))
        .with_texture(
            draw::Texture::make_tile_texture(&vec![0.8],&vec![4,4]),
            draw::TextureMapping{origin_verti : 0, frame_vertis : vec![1,3]}
        );
    build_test_walls(&build_shape, world);
}
pub fn build_test_level_4d(world: &mut World) {
    insert_wall(world,build_cube_4d(1.0).with_pos(&Vec4::new(3., 0., 0.,0.)));
    let build_shape: BuildFaceShape<Vec4>= BuildFaceShape::new(ShapeBuilder::<Vec3>::build_cube(2.0))
        .with_texture(
            draw::Texture::make_tile_texture(&vec![0.8],&vec![4,4,4]),
            draw::TextureMapping{origin_verti : 0, frame_vertis : vec![1,3,4]}
        );
    build_test_walls(&build_shape, world)
}
pub fn build_shapes_3d(world : &mut World) {
    build_lvl_1_3d(world);
    //build_test_level_3d(world);
    //build_test_face(world);
    init_player(world, Vec3::zero());
    init_cursor_3d(world);
}
pub fn build_shapes_4d(world : &mut World) {
    build_lvl_1_4d(world);
    //build_test_level_4d(world);
    init_player(world, Vec4::zero());
    init_cursor_4d(world);
    
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
            .with_pos(&(
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

    shapes.append(&mut walls1);

    //end walls
    
    let walls2 = iproduct!(axes.clone(),signs.iter())
        .map(|(i,sign)| cube.clone()
                .with_pos(&(V::one_hot(i)*(wall_length+corr_width)*(*sign)))
                .stretch(&(
                    V::one_hot(1)*(wall_height-corr_width) + V::ones()*corr_width
                    ))
                );
    shapes.append(&mut walls2.collect());
    //floors and ceilings
    let mut floors_long : Vec<Shape<V>> = iproduct!(axes.clone(),signs.iter())
        .map(|(i,sign)| cube.clone()
                .with_pos(&(V::one_hot(i)*(wall_length+corr_width)*(*sign)/2.0
                    - V::one_hot(1)*(wall_height + corr_width)/2.0
                    ))
                .stretch(&(V::one_hot(i)*(wall_length-corr_width) + V::ones()*corr_width
                    ))
                ).collect();
    let mut ceilings_long : Vec<Shape<V>> = floors_long.iter()
        .map(|block| block.clone().with_pos(&(
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
    shapes.push(cube.clone().with_pos(&(-V::one_hot(1)*(wall_height + corr_width)/2.0)));
    shapes
    
}
pub fn init_player<V: VectorTrait>(world: &mut World, pos: V) {
    let camera = Camera::new(pos);
    crate::player::build_player(world, camera);

}
pub fn init_cursor_3d(world: &mut World) {
    world.create_entity()
        .with(Cursor)
        .with(ShapeBuilder::<Vec2>::build_cube(0.03))
        .build();
}
pub fn init_cursor_4d(world: &mut World) {
    world.create_entity()
        .with(Cursor)
        .with(ShapeBuilder::<Vec3>::build_cube(0.03))
        .build();
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
                .with_pos(&(V::one_hot(axis)*dir*(wall_length - 0.5)))
        );
    }

}