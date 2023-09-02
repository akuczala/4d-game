use crate::coin::Coin;
use crate::components::{
    BBall, BBox, HasBBox, Shape, ShapeClipState, ShapeLabel, StaticCollider, Transform,
    Transformable,
};
use crate::config::Config;

use crate::draw::texture::shape_texture::ShapeTextureGeneric;
use crate::draw::texture::texture_builder::TextureBuilder;
use crate::draw::texture::ShapeTextureBuilder;
use crate::ecs_utils::Componentable;
use crate::geometry::shape::RefShapes;
use crate::graphics::colors::Color;
use crate::vector::VectorTrait;

use crate::geometry::transform::Scaling;
use specs::prelude::*;

#[derive(Clone)]
pub struct ShapeEntityBuilder<V, M> {
    shape_label: ShapeLabel,
    transform: Transform<V, M>,
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
            transform: Transform::identity(),
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
        self.transform.scale(Scaling::Vector(*scales));
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
        let c = self.make_components(&world.fetch::<Config>(), ref_shapes);
        world
            .create_entity()
            .with(c.bbox)
            .with(c.bball)
            .with(c.transform)
            .with(c.shape)
            .with(c.shape_label)
            .with(c.shape_texture_builder)
            .with(c.shape_texture)
            .with(c.shape_clip_state)
            .maybe_with(c.static_collider)
            .maybe_with(c.coin)
    }
    pub fn insert(
        self,
        e: Entity,
        lazy: &Read<LazyUpdate>,
        config: &Config,
        ref_shapes: &RefShapes<V>,
    ) {
        let c = self.make_components(config, ref_shapes);
        lazy.insert(e, c.bbox);
        lazy.insert(e, c.bball);
        lazy.insert(e, c.transform);
        lazy.insert(e, c.shape);
        lazy.insert(e, c.shape_texture_builder);
        lazy.insert(e, c.shape_texture);
        lazy.insert(e, c.shape_clip_state);
        lazy.insert(e, c.shape_label);
        if let Some(c) = c.static_collider {
            lazy.insert(e, c)
        };
        if let Some(c) = c.coin {
            lazy.insert(e, c)
        };
    }
    fn make_components(
        self,
        config: &Config,
        ref_shapes: &RefShapes<V>,
    ) -> ComponentsToInsert<V, V::M, V::SubV> {
        let Self {
            shape_label,
            transform,
            shape_texture_builder,
            static_collider,
            coin,
        } = self;
        let ref_shape = ref_shapes.get_unwrap(&shape_label);
        let mut shape = ref_shape.clone();
        shape.update_from_ref(ref_shape, &transform);
        let shape_texture =
            shape_texture_builder
                .clone()
                .build::<V>(&config.into(), ref_shape, &shape, &transform);
        ComponentsToInsert {
            shape_label,
            bbox: shape.calc_bbox(),
            bball: BBall::new(&shape.verts, transform.pos),
            transform,
            shape,
            shape_texture_builder,
            shape_texture,
            shape_clip_state: ShapeClipState::<V>::default(),
            static_collider,
            coin,
        }
    }
}
impl<V: VectorTrait> Transformable<V> for ShapeEntityBuilderV<V> {
    fn transform(&mut self, transformation: Transform<V, V::M>) {
        self.transform = self.transform.with_transform(transformation);
    }
}
struct ComponentsToInsert<V, M, U> {
    shape_label: ShapeLabel,
    bbox: BBox<V>,
    bball: BBall<V>,
    transform: Transform<V, M>,
    shape: Shape<V>,
    shape_texture_builder: ShapeTextureBuilder,
    shape_texture: ShapeTextureGeneric<V, M, U>,
    shape_clip_state: ShapeClipState<V>,
    static_collider: Option<StaticCollider>,
    coin: Option<Coin>,
}
