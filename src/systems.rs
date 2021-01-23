use std::marker::PhantomData;

pub type SysTypeV<V> = PhantomData<V>;
pub type SysTypeVS<V,S> = PhantomData<(V,S)>;

//for easy import of all systems
pub use crate::draw::{
	VisibilitySystem,
	CalcShapesLinesSystem,
	TransformDrawLinesSystem,
	DrawCursorSystem,
};
pub use crate::clipping::{InFrontSystem};
pub use crate::input::{UpdateCameraSystem,PrintDebugSystem};
pub use crate::player::{ShapeTargetingSystem};
pub use crate::collide::{
	PlayerCollisionDetectionSystem,
	PlayerStaticCollisionSystem,
	MovePlayerSystem,
	UpdatePlayerBBox,
	UpdateBBoxSystem
};
pub use crate::coin::{
	CoinSpinningSystem,
	PlayerCoinCollisionSystem
};
pub use crate::cleanup::{ShapeCleanupSystem};
