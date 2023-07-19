use crate::components::{
    BBall, Convex, HasBBox, Shape, ShapeClipState, ShapeLabel, ShapeType, SingleFace,
    StaticCollider, Transform, Transformable,
};
use crate::draw::{FaceTexture, ShapeTexture, Texture, TextureMapping};
use crate::ecs_utils::Componentable;
use crate::geometry::shape::{buildshapes, RefShapes};
use crate::graphics::colors::Color;
use crate::saveload::SaveMarker;
use crate::vector::VectorTrait;

use crate::geometry::transform::Scaling;
use specs::prelude::*;
use specs::saveload::MarkedBuilder;

#[derive(Clone)]
pub struct ShapeEntityBuilder<V, U, M> {
    pub shape: Shape<V>, // remove this field?
    shape_label: ShapeLabel,
    pub transformation: Transform<V, M>,
    pub shape_texture: ShapeTexture<U>,
    static_collider: Option<StaticCollider>,
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
            shape_texture: ShapeTexture::new_default(ref_shape.verts.len()),
            static_collider: None,
        }
    }
    pub fn with_texture(mut self, texture: ShapeTexture<V::SubV>) -> Self {
        self.shape_texture = texture;
        self
    }
    pub fn with_face_texture(mut self, face_texture: FaceTexture<V::SubV>) -> Self {
        self.shape_texture = self.shape_texture.with_texture(face_texture);
        self
    }
    pub fn with_texturing_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&Shape<V>) -> ShapeTexture<V::SubV>,
    {
        self.shape_texture = f(&self.shape);
        self
    }
    pub fn with_color(mut self, color: Color) -> Self {
        self.shape_texture = self.shape_texture.with_color(color);

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
    pub fn build(self, world: &mut World) -> EntityBuilder {
        let Self {
            mut shape,
            shape_label,
            transformation,
            shape_texture,
            static_collider,
        } = self;
        shape.update_from_ref(&shape.clone(), &transformation);
        if let ShapeType::SingleFace(ref mut single_face) = shape.shape_type {
            single_face.update(&shape.verts, &shape.faces)
        }
        world
            .create_entity()
            .with(shape.calc_bbox())
            .with(BBall::new(&shape.verts, transformation.pos))
            .with(transformation)
            .with(shape)
            .with(shape_label)
            .with(shape_texture)
            .with(ShapeClipState::<V>::default())
            .maybe_with(static_collider)
            .marked::<SaveMarker>()
    }
    pub fn insert(self, e: Entity, lazy: &Read<LazyUpdate>) {
        let Self {
            mut shape,
            shape_label,
            transformation,
            shape_texture,
            static_collider,
        } = self;
        shape.update_from_ref(&shape.clone(), &transformation);
        if let ShapeType::SingleFace(ref mut single_face) = shape.shape_type {
            single_face.update(&shape.verts, &shape.faces)
        }
        lazy.insert(e, shape.calc_bbox());
        lazy.insert(e, BBall::new(&shape.verts, transformation.pos));
        lazy.insert(e, transformation);
        lazy.insert(e, shape);
        lazy.insert(e, shape_texture);
        lazy.insert(e, ShapeClipState::<V>::default());
        lazy.insert(e, shape_label);
        if let Some(c) = static_collider {
            lazy.insert(e, c)
        };

        // TODO: mark with SaveMarker
    }
}
impl<V: VectorTrait> Transformable<V> for ShapeEntityBuilderV<V> {
    fn transform(&mut self, transformation: Transform<V, V::M>) {
        self.transformation = self.transformation.with_transform(transformation);
    }
}
