use std::marker::PhantomData;

use crate::components::{
    BBall, Convex, HasBBox, Shape, ShapeClipState, ShapeLabel, ShapeType, SingleFace,
    StaticCollider, Transform, Transformable,
};
use crate::config::Config;
use crate::draw::texture::texture_builder::{TextureBuilder, TextureBuilderConfig};
use crate::draw::texture::{FaceTextureBuilder, ShapeTextureBuilder};
use crate::draw::{FaceTexture, ShapeTexture, Texture, TextureMapping};
use crate::ecs_utils::Componentable;
use crate::geometry::shape::{buildshapes, RefShapes};
use crate::graphics::colors::Color;
use crate::vector::VectorTrait;

use crate::geometry::transform::Scaling;
use specs::prelude::*;
use specs::saveload::MarkedBuilder;

#[derive(Clone)]
pub struct ShapeEntityBuilder<V, U, M> {
    pub shape: Shape<V>, // remove this field?
    shape_label: ShapeLabel,
    pub transformation: Transform<V, M>,
    pub shape_texture_builder: ShapeTextureBuilder,
    static_collider: Option<StaticCollider>,
    ph: PhantomData<U>, // TODO: remove if unnecessary
}

//shorthand
pub type ShapeEntityBuilderV<V> =
    ShapeEntityBuilder<V, <V as VectorTrait>::SubV, <V as VectorTrait>::M>;

impl<V: VectorTrait> ShapeEntityBuilderV<V> {
    pub fn new_from_ref_shape(ref_shapes: &RefShapes<V>, label: ShapeLabel) -> Self {
        let ref_shape = ref_shapes.get_unwrap(&label);
        Self {
            shape: ref_shape.clone(),
            shape_label: label,
            transformation: Transform::identity(),
            shape_texture_builder: ShapeTextureBuilder::new_default(ref_shape.verts.len()),
            static_collider: None,
            ph: PhantomData::<V::SubV>,
        }
    }
    pub fn with_texture(mut self, texture: ShapeTextureBuilder) -> Self {
        self.shape_texture_builder = texture;
        self
    }
    pub fn with_face_texture(mut self, face_texture: FaceTextureBuilder) -> Self {
        self.shape_texture_builder = self.shape_texture_builder.with_texture(face_texture);
        self
    }
    pub fn with_texturing_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&Shape<V>) -> ShapeTextureBuilder,
    {
        self.shape_texture_builder = f(&self.shape);
        self
    }
    pub fn with_color(mut self, color: Color) -> Self {
        self.shape_texture_builder = self.shape_texture_builder.with_color(color);

        self
    }
    pub fn with_collider(mut self, static_collider: Option<StaticCollider>) -> Self {
        self.static_collider = static_collider;
        self
    }
    pub fn stretch(mut self, scales: &V) -> Self {
        self.transformation.scale(Scaling::Vector(*scales));
        self
    }
}
impl<V> ShapeEntityBuilderV<V>
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    // TODO: use macro to specify components in both methods?
    pub fn build(self, world: &mut World) -> EntityBuilder {
        let Self {
            mut shape,
            shape_label,
            transformation,
            shape_texture_builder,
            static_collider,
            ph: _,
        } = self;
        shape.update_from_ref(&shape.clone(), &transformation);
        let shape_texture =
            make_shape_texture::<V::SubV>(&world.fetch::<Config>(), shape_texture_builder.clone());
        world
            .create_entity()
            .with(shape.calc_bbox())
            .with(BBall::new(&shape.verts, transformation.pos))
            .with(transformation)
            .with(shape)
            .with(shape_label)
            .with(shape_texture_builder)
            .with(shape_texture)
            .with(ShapeClipState::<V>::default())
            .maybe_with(static_collider)
    }
    pub fn insert(self, e: Entity, lazy: &Read<LazyUpdate>, config: &Config) {
        let Self {
            mut shape,
            shape_label,
            transformation,
            shape_texture_builder,
            static_collider,
            ph: _,
        } = self;
        shape.update_from_ref(&shape.clone(), &transformation);
        let shape_texture = make_shape_texture::<V::SubV>(config, shape_texture_builder.clone());
        lazy.insert(e, shape.calc_bbox());
        lazy.insert(e, BBall::new(&shape.verts, transformation.pos));
        lazy.insert(e, transformation);
        lazy.insert(e, shape);
        lazy.insert(e, shape_texture_builder);
        lazy.insert(e, shape_texture);
        lazy.insert(e, ShapeClipState::<V>::default());
        lazy.insert(e, shape_label);
        if let Some(c) = static_collider {
            lazy.insert(e, c)
        };
    }
    pub fn load(self, e: Entity, lazy: &Read<LazyUpdate>, config: &Config) {
        let Self {
            mut shape,
            shape_label: _,
            transformation,
            shape_texture_builder,
            static_collider: _,
            ph: _,
        } = self;
        shape.update_from_ref(&shape.clone(), &transformation);
        let shape_texture = make_shape_texture::<V::SubV>(config, shape_texture_builder);
        lazy.insert(e, shape.calc_bbox());
        lazy.insert(e, BBall::new(&shape.verts, transformation.pos));
        lazy.insert(e, shape);
        lazy.insert(e, shape_texture);
        lazy.insert(e, ShapeClipState::<V>::default());
    }
}
impl<V: VectorTrait> Transformable<V> for ShapeEntityBuilderV<V> {
    fn transform(&mut self, transformation: Transform<V, V::M>) {
        self.transformation = self.transformation.with_transform(transformation);
    }
}

fn make_shape_texture<U: VectorTrait>(
    config: &Config,
    builder: ShapeTextureBuilder,
) -> ShapeTexture<U> {
    builder.build::<U>(&config.into())
}
