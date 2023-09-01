use crate::coin::Coin;
use crate::components::{
    BBall, HasBBox, Shape, ShapeClipState, ShapeLabel, StaticCollider, Transform, Transformable,
};
use crate::config::Config;

use crate::draw::texture::texture_builder::TextureBuilder;
use crate::draw::texture::ShapeTextureBuilder;
use crate::draw::ShapeTexture;
use crate::ecs_utils::Componentable;
use crate::geometry::shape::RefShapes;
use crate::graphics::colors::Color;
use crate::vector::VectorTrait;

use crate::geometry::transform::Scaling;
use specs::prelude::*;

#[derive(Clone)]
pub struct ShapeEntityBuilder<V, M> {
    shape_label: ShapeLabel,
    transformation: Transform<V, M>,
    shape_texture_builder: ShapeTextureBuilder,
    static_collider: Option<StaticCollider>,
    coin: Option<Coin>,
}

//shorthand
pub type ShapeEntityBuilderV<V> = ShapeEntityBuilder<V, <V as VectorTrait>::M>;

impl<V: VectorTrait> ShapeEntityBuilderV<V> {
    pub fn new(label: ShapeLabel) -> Self {
        Self {
            shape_label: label,
            transformation: Transform::identity(),
            shape_texture_builder: ShapeTextureBuilder::default(),
            static_collider: None,
            coin: None,
        }
    }
    pub fn with_texture(mut self, texture: ShapeTextureBuilder) -> Self {
        self.shape_texture_builder = texture;
        self
    }
    pub fn with_face_texture(mut self, texture: TextureBuilder) -> Self {
        self.shape_texture_builder = self.shape_texture_builder.with_texture(texture);
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
    pub fn build<'a>(self, ref_shapes: &RefShapes<V>, world: &'a mut World) -> EntityBuilder<'a> {
        let Self {
            shape_label,
            transformation,
            shape_texture_builder,
            static_collider,
            coin,
        } = self;
        let ref_shape = ref_shapes.get_unwrap(&shape_label);
        let mut shape = ref_shape.clone();
        shape.update_from_ref(ref_shape, &transformation);
        let shape_texture = make_shape_texture::<V>(
            &world.fetch::<Config>(),
            shape_texture_builder.clone(),
            ref_shape,
            &shape,
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
    pub fn insert(
        self,
        e: Entity,
        lazy: &Read<LazyUpdate>,
        config: &Config,
        ref_shapes: &RefShapes<V>,
    ) {
        let Self {
            shape_label,
            transformation,
            shape_texture_builder,
            static_collider,
            coin,
        } = self;
        let ref_shape = ref_shapes.get_unwrap(&shape_label);
        let mut shape = ref_shape.clone();
        shape.update_from_ref(ref_shape, &transformation);
        let shape_texture =
            make_shape_texture::<V>(config, shape_texture_builder.clone(), ref_shape, &shape);
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
    ref_shape: &Shape<V>,
    shape: &Shape<V>,
) -> ShapeTexture<V> {
    builder.build::<V>(&config.into(), ref_shape, shape)
}
