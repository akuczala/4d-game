use crate::vector::VectorTrait;
use crate::components::{Shape,ShapeType,Transform,Transformable,Convex,BBall,ShapeClipState,HasBBox};
use crate::draw::{Texture,TextureMapping};
use crate::geometry::shape::buildshapes;
use crate::graphics::colors::Color;

use specs::prelude::*;
use crate::geometry::transform::Scaling;

#[derive(Clone)]
pub struct ShapeEntityBuilder<V: VectorTrait> {
    pub shape: Shape<V>,
    shape_type: ShapeType<V>,
    pub transformation: Transform<V>,
    texture_info: Option<(Texture<V::SubV>, TextureMapping)>
}
impl<'a,V: VectorTrait> ShapeEntityBuilder<V> {
    pub fn new_face_shape(sub_shape: Shape<V::SubV>, two_sided: bool) -> Self {
        let (shape, single_face) = buildshapes::convex_shape_to_face_shape(sub_shape, two_sided);
        Self{
            shape,
            shape_type: ShapeType::SingleFace(single_face),
            transformation: Transform::identity(),
            texture_info: None
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
    pub fn with_texture(mut self, texture: Texture<V::SubV>, texture_mapping: TextureMapping) -> Self {
        self.texture_info = Some((texture, texture_mapping));
        self
    }
    pub fn with_color(mut self, color: Color) -> Self {
        self.texture_info = match self.texture_info {
            Some((texture, texture_mapping)) =>
                Some((texture.set_color(color), texture_mapping)),
            None => Some((Texture::default().set_color(color), Default::default()))
        };

        self
    }
    pub fn stretch(mut self, scales : &V) -> Self {
        self.transformation.scale(Scaling::Vector(*scales));
        self
    }
    pub fn build(self, world: &mut World) -> EntityBuilder {
        let Self{
            mut shape,
            mut shape_type,
            transformation,
            texture_info,} = self;
        shape.update_from_ref(&shape.clone(),&transformation);
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
    pub fn insert(self, e: Entity, lazy: &Read<LazyUpdate>) {
        let Self{
            mut shape,
            mut shape_type,
            transformation,
            texture_info,} = self;
        shape.update_from_ref(&shape.clone(),&transformation);
        if let Some((texture, texture_mapping)) = texture_info {
            for face in shape.faces.iter_mut() {
                face.set_texture(texture.clone(), texture_mapping.clone());
            }
        }
        match shape_type {
            ShapeType::SingleFace(ref mut single_face) => {single_face.update(&shape)},
            _ => (),
        }
        lazy.insert(e, shape.calc_bbox());
        lazy.insert(e, BBall::new(&shape.verts, transformation.pos));
        lazy.insert(e, transformation);
        lazy.insert(e, shape_type);
        lazy.insert(e, shape);
        lazy.insert(e, ShapeClipState::<V>::default());
    }
}
impl<V: VectorTrait> Transformable<V> for ShapeEntityBuilder<V> {
    fn transform(&mut self, transformation: Transform<V>) {
        self.transformation = self.transformation.with_transform(transformation);
    }
}