use crate::vector::VectorTrait;
use crate::components::{Shape,ShapeType,Transform,Transformable,Convex,BBall,ShapeClipState,HasBBox, ShapeLabel};
use crate::draw::{Texture,TextureMapping, ShapeTexture, FaceTexture};
use crate::geometry::shape::{buildshapes, RefShapes};
use crate::graphics::colors::Color;

use specs::prelude::*;
use crate::geometry::transform::Scaling;

#[derive(Clone)]
pub struct ShapeEntityBuilder<V: VectorTrait> {
    pub shape: Shape<V>, // remove this field?
    shape_type: ShapeType<V>,
    shape_label: Option<ShapeLabel>, // TODO: make this mandatory
    pub transformation: Transform<V>,
    pub shape_texture: ShapeTexture<V>,
}
impl<'a,V: VectorTrait> ShapeEntityBuilder<V> {
    pub fn new_face_shape(sub_shape: Shape<V::SubV>, two_sided: bool) -> Self {
        let (shape, single_face) = buildshapes::convex_shape_to_face_shape(sub_shape, two_sided);
        let shape_texture = ShapeTexture::new_default(shape.verts.len());
        Self{
            shape,
            shape_type: ShapeType::SingleFace(single_face),
            shape_label: None,
            transformation: Transform::identity(),
            shape_texture
        }
    }
    pub fn new_convex_shape(shape: Shape<V>) -> Self {
        let convex = Convex::new(&shape);
        let shape_texture = ShapeTexture::new_default(shape.verts.len());
        Self{
            shape,
            shape_type: ShapeType::Convex(convex),
            shape_label: None,
            transformation: Transform::identity(),
            shape_texture
        }
    }
    pub fn convex_from_ref_shape(ref_shapes: &RefShapes<V>, label: ShapeLabel) -> Self {
        let ref_shape = ref_shapes.get_unwrap(&label);
        Self {
            shape: ref_shape.clone(),
            shape_type: ShapeType::Convex(Convex::new(ref_shape)),
            shape_label: Some(label),
            transformation: Transform::identity(),
            shape_texture: ShapeTexture::new_default(ref_shape.verts.len())
        }
    }
    pub fn with_texture(mut self, texture: ShapeTexture<V>) -> Self {

		self.shape_texture = texture;
		self
	}
    pub fn with_face_texture(mut self, face_texture: FaceTexture<V>) -> Self {
        self.shape_texture = self.shape_texture.with_texture(face_texture);
        self
    }
    pub fn with_texturing_fn<F>(mut self, f: F) -> Self
    where F: Fn(&Shape<V>) -> ShapeTexture<V> {
        self.shape_texture = f(&self.shape);
        self
    }
    pub fn with_color(mut self, color: Color) -> Self {
        self.shape_texture = self.shape_texture.with_color(color);

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
            shape_label,
            transformation,
            shape_texture 
        } = self;
        shape.update_from_ref(&shape.clone(),&transformation);
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
            .maybe_with(shape_label)
            .with(shape_texture)
            .with(ShapeClipState::<V>::default())
    }
    pub fn insert(self, e: Entity, lazy: &Read<LazyUpdate>) {
        let Self{
            mut shape,
            mut shape_type,
            shape_label,
            transformation,
            shape_texture
        } = self;
        shape.update_from_ref(&shape.clone(),&transformation);
        match shape_type {
            ShapeType::SingleFace(ref mut single_face) => {single_face.update(&shape)},
            _ => (),
        }
        lazy.insert(e, shape.calc_bbox());
        lazy.insert(e, BBall::new(&shape.verts, transformation.pos));
        lazy.insert(e, transformation);
        lazy.insert(e, shape_type);
        lazy.insert(e, shape);
        lazy.insert(e, shape_texture);
        lazy.insert(e, ShapeClipState::<V>::default());
        if let Some(label) = shape_label {
            lazy.insert(e, label)
        }
    }
}
impl<V: VectorTrait> Transformable<V> for ShapeEntityBuilder<V> {
    fn transform(&mut self, transformation: Transform<V>) {
        self.transformation = self.transformation.with_transform(transformation);
    }
}