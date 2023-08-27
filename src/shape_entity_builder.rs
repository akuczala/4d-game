use crate::coin::Coin;
use crate::components::{
    BBall, HasBBox, Shape, ShapeClipState, ShapeLabel, StaticCollider, Transform, Transformable,
};
use crate::config::Config;

use crate::draw::texture::{FaceTextureBuilder, ShapeTextureBuilder};
use crate::draw::ShapeTexture;
use crate::ecs_utils::Componentable;
use crate::geometry::shape::RefShapes;
use crate::graphics::colors::Color;
use crate::vector::VectorTrait;

use crate::geometry::transform::Scaling;
use specs::prelude::*;

#[derive(Clone)]
pub struct ShapeEntityBuilder<V, M> {
    ref_shape: Shape<V>, // this field is only needed for with_texturing_fn; we don't really need until build
    shape_label: ShapeLabel,
    pub transformation: Transform<V, M>,
    pub shape_texture_builder: ShapeTextureBuilder,
    static_collider: Option<StaticCollider>,
    coin: Option<Coin>,
}

//shorthand
pub type ShapeEntityBuilderV<V> = ShapeEntityBuilder<V, <V as VectorTrait>::M>;

impl<V: VectorTrait> ShapeEntityBuilderV<V> {
    pub fn new_from_ref_shape(ref_shapes: &RefShapes<V>, label: ShapeLabel) -> Self {
        let ref_shape = ref_shapes.get_unwrap(&label);
        Self {
            ref_shape: ref_shape.clone(),
            shape_label: label,
            transformation: Transform::identity(),
            shape_texture_builder: ShapeTextureBuilder::new_default(ref_shape.faces.len()),
            static_collider: None,
            coin: None,
        }
    }
    // TODO: this is a temporary hack until ref_shape is not part of the struct
    pub fn n_faces(&self) -> usize {
        self.ref_shape.faces.len()
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
        self.shape_texture_builder = f(&self.ref_shape);
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
    pub fn with_coin(mut self, coin: Option<Coin>) -> Self {
        self.coin = coin;
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
            ref_shape,
            shape_label,
            transformation,
            shape_texture_builder,
            static_collider,
            coin,
        } = self;
        let mut shape = ref_shape.clone();
        shape.update_from_ref(&ref_shape, &transformation);
        let shape_texture = make_shape_texture::<V>(
            &world.fetch::<Config>(),
            shape_texture_builder.clone(),
            &ref_shape,
        );
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
            .maybe_with(coin)
    }
    pub fn insert(self, e: Entity, lazy: &Read<LazyUpdate>, config: &Config) {
        let Self {
            ref_shape,
            shape_label,
            transformation,
            shape_texture_builder,
            static_collider,
            coin,
        } = self;
        let mut shape = ref_shape.clone();
        shape.update_from_ref(&ref_shape, &transformation);
        let shape_texture =
            make_shape_texture::<V>(config, shape_texture_builder.clone(), &ref_shape);
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
        if let Some(c) = coin {
            lazy.insert(e, c)
        };
    }
}
impl<V: VectorTrait> Transformable<V> for ShapeEntityBuilderV<V> {
    fn transform(&mut self, transformation: Transform<V, V::M>) {
        self.transformation = self.transformation.with_transform(transformation);
    }
}

fn make_shape_texture<V: VectorTrait>(
    config: &Config,
    builder: ShapeTextureBuilder,
    shape: &Shape<V>,
) -> ShapeTexture<V> {
    builder.build::<V>(&config.into(), shape)
}
