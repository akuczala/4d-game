use crate::components::{Cursor,Transform,BBox,BBall};
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
use glium::framebuffer::ColorAttachment::Texture;

pub fn insert_wall<V : VectorTrait>(world : &mut World, shape_builder : BuildShape<V>) {
    shape_builder.build(world)
        .with(StaticCollider)
        .build();
}
pub fn insert_coin<V : VectorTrait>(world : &mut World, shape_builder : BuildShape<V>) {
    shape_builder.build(world)
        .with(Coin)
        .build();
}
#[derive(Clone)]
pub struct BuildShape<V: VectorTrait> {
    shape: Shape<V>,
    shape_type: ShapeType<V>,
    transformation: Transform<V>,
    texture_info: Option<(draw::Texture<V::SubV>, draw::TextureMapping)>,
}
impl<V: VectorTrait> BuildShape<V> {
    pub fn new_face_shape(sub_shape: Shape<V::SubV>) -> Self {
        let (mut shape, mut single_face) = buildshapes::convex_shape_to_face_shape(sub_shape);
        Self{
            shape,
            shape_type: ShapeType::SingleFace(single_face),
            transformation: Transform::identity(),
            texture_info: None,
        }
    }
    pub fn new_convex_shape(shape: Shape<V>) -> Self {
        let convex = Convex::new(&shape);
        Self{
            shape,
            shape_type: ShapeType::Convex(convex),
            transformation: Transform::identity(),
            texture_info: None,
        }
    }
    pub fn with_texture(mut self, texture: draw::Texture<V::SubV>, texture_mapping: draw::TextureMapping) -> Self {
        self.texture_info = Some((texture, texture_mapping));
        self
    }
    pub fn with_color(mut self, color: Color) -> Self {
        self.texture_info = match self.texture_info {
            Some((texture, texture_mapping)) =>
                Some((texture.set_color(color), texture_mapping)),
            None => Some((draw::Texture::default().set_color(color), Default::default()))
        };

        self
    }
    pub fn stretch(mut self, scales : &V) -> Self {
        self.shape = self.shape.stretch(scales);
        self
    }
    pub fn build(self, world: &mut World) -> EntityBuilder {
        let Self{mut shape, mut shape_type, transformation, texture_info} = self;
        shape = shape.with_transform(transformation);
        if let Some((texture, texture_mapping)) = texture_info {
            for face in shape.faces.iter_mut() {
                face.set_texture(texture.clone(), texture_mapping.clone());
            }
        }
        match shape_type {
            ShapeType::SingleFace(ref mut single_face) => {single_face.update(&shape)},
            _ => (),
        }
        world.create_entity()
            .with(shape.calc_bbox())
            .with(BBall::new(&shape.verts, transformation.pos))
            .with(transformation)
            .with(shape_type)
            .with(shape)
            .with(ShapeClipState::<V>::default())
    }
}
impl<V: VectorTrait> Transformable<V> for BuildShape<V> {
    fn set_identity(mut self) -> Self {
        self.transformation = Transform::identity();
        self
    }
    fn transform(&mut self, transformation: Transform<V>) {
        self.transformation = self.transformation.with_transform(transformation);
    }
}

fn build_test_walls<V: VectorTrait>(build_shape: &BuildShape<V>, world: &mut World) {
    let theta = (PI/6.0);
    let cos = theta.cos();
    let sin = theta.sin();
    build_shape.clone()
        .with_translation(V::one_hot(-1)*(-1.0 - cos) + V::one_hot(1)*(sin - 1.0))
        .with_rotation(-1, 1, PI/2.0 - theta)
        .with_color(RED)
        .build(world).build();
    build_shape.clone()
        .with_translation(V::one_hot(-1)*1.0)
        .with_rotation(0,-1,PI)
        .with_color(GREEN)
        .build(world).build();
    build_shape.clone()
        .with_translation(V::one_hot(0)*1.0)
        .with_rotation(0,-1,PI/2.)
        .with_color(ORANGE)
        .build(world).build();
    build_shape.clone()
        .with_translation(-V::one_hot(0)*1.0)
        .with_rotation(0,-1,3.0*PI/2.)
        .with_color(CYAN)
        .build(world).build();
    let floor = build_shape.clone()
        .with_translation(-V::one_hot(1)*1.0)
        .with_rotation(-1,1,PI/2.)
        .with_color(BLUE);
    floor.clone()
        .with_translation(V::one_hot(1)*(2.0*sin) - V::one_hot(-1)*(2.0 + 2.0*cos))
        .build(world).build();
    floor.build(world);
    build_shape.clone()
        .with_translation(V::one_hot(1)*1.0)
        .with_rotation(-1,1,-PI/2.)
        .with_color(YELLOW)
        .build(world).build();
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

pub fn build_corridor_cross<V : VectorTrait>(cube : &Shape<V>, wall_length : Field) -> Vec<BuildShape<V>> {

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
    
    let mut shape_builders : Vec<BuildShape<V>> = Vec::new();
    //corridor walls
    let mut walls1 : Vec<BuildShape<V>> = iproduct!(signs.iter(),signs.iter(),axis_pairs.iter())
        .map(|(s1,s2,(ax1,ax2))|
            BuildShape::new_convex_shape(cube.clone())
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
        apply_texture(&mut builder.shape);
    }

    shape_builders.append(&mut walls1);

    //end walls
    
    let walls2 = iproduct!(axes.clone(),signs.iter())
        .map(|(i,sign)|
            BuildShape::new_convex_shape(cube.clone())
                .with_translation(V::one_hot(i)*(wall_length+corr_width)*(*sign))
                .stretch(&(
                    V::one_hot(1)*(wall_height-corr_width) + V::ones()*corr_width
                    ))
                );
    shape_builders.append(&mut walls2.collect());
    //floors and ceilings
    let mut floors_long : Vec<BuildShape<V>> = iproduct!(axes.clone(),signs.iter())
        .map(|(i,sign)|
            BuildShape::new_convex_shape(cube.clone())
                .with_translation(V::one_hot(i)*(wall_length+corr_width)*(*sign)/2.0
                    - V::one_hot(1)*(wall_height + corr_width)/2.0
                    )
                .stretch(&(V::one_hot(i)*(wall_length-corr_width) + V::ones()*corr_width
                    ))
                ).collect();
    let mut ceilings_long : Vec<BuildShape<V>> = floors_long.iter()
        .map(|block| block.clone().with_translation(
            V::one_hot(1)*(wall_height+corr_width)
        ))
        .collect();

    for builder in &mut floors_long {
        apply_texture(&mut builder.shape);
    }
    for builder in &mut ceilings_long {
        apply_texture(&mut builder.shape);
    }

    shape_builders.append(&mut floors_long);
    shape_builders.append(&mut ceilings_long);
    //center floor
    shape_builders.push(
        BuildShape::new_convex_shape(cube.clone())
            .with_translation(-V::one_hot(1)*(wall_height + corr_width)/2.0)
    );
    shape_builders
    
}
pub fn init_player<V: VectorTrait>(world: &mut World, pos: V) {
    let transform = Transform::identity().with_translation(pos);
    crate::player::build_player(world, &transform);

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
    let walls : Vec<BuildShape<V>> = build_corridor_cross(&color_cube(cube),wall_length);

    for wall in walls.into_iter() {
        insert_wall(world,wall)
    }
    //let (m,n) = (4,4);
    //let mut duocylinder = buildshapes::build_duoprism_4d([1.0,1.0],[[0,1],[2,3]],[m,n])
    for (axis,dir) in iproduct!(match V::DIM {3 => vec![0,2], 4 => vec![0,2,3], _ => panic!("Invalid dimension")},vec![-1.,1.]) {
        insert_coin(world,
                    BuildShape::new_convex_shape(coin.clone())
                .with_translation(V::one_hot(axis)*dir*(wall_length - 0.5))
        );
    }

}