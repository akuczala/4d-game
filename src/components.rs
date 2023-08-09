use specs::{Component, FlaggedStorage, HashMapStorage, VecStorage};

use crate::coin::Coin;
pub use crate::draw::draw_line_collection::DrawLineCollection;

use crate::draw::texture::ShapeTextureBuilder;
pub use crate::draw::ShapeTexture;
use crate::ecs_utils::Componentable;
//resources
pub use crate::player::Player;
//components
pub use crate::camera::Camera;
pub use crate::collide::{bbox::BBox, bbox::HasBBox, InPlayerCell, MoveNext, StaticCollider};
pub use crate::draw::clipping::{bball::BBall, ClipState, ShapeClipState};
pub use crate::draw::DrawLineList;
pub use crate::geometry::{
    shape::RefShapes,
    shape::{Convex, Shape, ShapeLabel, ShapeType, SingleFace},
    transform::{Transform, Transformable},
};
pub use crate::player::{Cursor, Heading, MaybeSelected, MaybeTarget, Selected};

type DefaultStorage<V> = VecStorage<V>;

impl<V: Componentable> Component for BBox<V> {
    type Storage = FlaggedStorage<Self, DefaultStorage<Self>>;
}

impl<V: Componentable> Component for BBall<V> {
    type Storage = DefaultStorage<Self>;
}

impl<V: Componentable> Component for Shape<V> {
    type Storage = FlaggedStorage<Self, DefaultStorage<Self>>;
}
impl<V: Componentable> Component for ShapeClipState<V> {
    type Storage = DefaultStorage<Self>;
}
impl Component for ShapeTextureBuilder {
    type Storage = DefaultStorage<Self>;
}
impl<U: Componentable> Component for ShapeTexture<U> {
    type Storage = DefaultStorage<Self>;
}
impl<V: Componentable, M: Componentable> Component for Transform<V, M> {
    type Storage = FlaggedStorage<Self, DefaultStorage<Self>>;
}
impl<V: Componentable> Component for Camera<V> {
    type Storage = DefaultStorage<Self>;
}
impl Component for Coin {
    type Storage = DefaultStorage<Self>;
}

impl<V: Componentable> Component for MoveNext<V> {
    type Storage = DefaultStorage<Self>;
}

impl<V: Componentable> Component for MaybeTarget<V> {
    type Storage = HashMapStorage<Self>;
}

impl<V: Componentable> Component for DrawLineCollection<V> {
    type Storage = HashMapStorage<Self>;
}

impl<M: Componentable> Component for Heading<M> {
    type Storage = HashMapStorage<Self>;
}
