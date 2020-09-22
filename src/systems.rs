//for easy import of all systems
pub use crate::draw::{
	VisibilitySystem,
	CalcShapesLinesSystem,
	TransformDrawLinesSystem
};
pub use crate::clipping::{InFrontSystem};
pub use crate::input::{UpdateCameraSystem,PrintDebugSystem};
pub use crate::collide::{
	PlayerCollisionDetectionSystem,
	PlayerStaticCollisionSystem,
	MovePlayerSystem,
	UpdatePlayerBBox,
};
pub use crate::coin::{
	CoinSpinningSystem,
	PlayerCoinCollisionSystem
};
pub use crate::cleanup::{ShapeCleanupSystem};