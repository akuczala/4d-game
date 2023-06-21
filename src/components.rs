

use specs::{FlaggedStorage, VecStorage, Component, HashMapStorage};

use crate::coin::Coin;
use crate::draw::ShapeTexture;
use crate::ecs_utils::Componentable;
//resources
pub use crate::player::{Player};
//components
pub use crate::camera::{Camera};
pub use crate::player::{MaybeTarget,MaybeSelected,Cursor,Selected};
pub use crate::geometry::{
    transform::{Transform,Transformable},
    shape::{Shape,ShapeType,ShapeTypeTrait,Convex,SingleFace,ShapeLabel}
};
pub use crate::draw::{DrawLineList};
pub use crate::draw::clipping::{ClipState, ShapeClipState, bball::BBall};
pub use crate::collide::{InPlayerCell, bbox::BBox, bbox::HasBBox,MoveNext,StaticCollider};
use crate::vector::VectorTrait;

type DefaultStorage<V> = VecStorage<V>;

impl<V: Componentable> Component for BBox<V> {
	type Storage = DefaultStorage<Self>;
}

impl<V: Componentable> Component for BBall<V> {
	type Storage = DefaultStorage<Self>;
}

impl<V: Componentable> Component for Shape<V> {
    type Storage = FlaggedStorage<Self, DefaultStorage<Self>>;
}

impl<V: Componentable> Component for ShapeType<V> {
    type Storage = FlaggedStorage<Self, DefaultStorage<Self>>;
}
impl<V: Componentable> Component for ShapeClipState<V> {
	type Storage = DefaultStorage<Self>;
}
impl<V, U, M> Component for ShapeTexture<V>
where
    V: VectorTrait<M = M, SubV = U> + Componentable,
    U: Componentable,
    M: Componentable
{
	type Storage = DefaultStorage<Self>;
}
impl<V: Componentable, M: Componentable> Component for Transform<V, M> {
    type Storage = FlaggedStorage<Self, DefaultStorage<Self>>;
}
impl<V: Componentable, M: Componentable> Component for Camera<V, M> {
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

impl<V: Componentable> Component for MaybeSelected<V> {
    type Storage = HashMapStorage<Self>;
}