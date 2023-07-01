mod shape_systems;
//for easy import of all systems
pub use crate::draw::systems::*;
pub use self::shape_systems::{
	UpdateBBallSystem,
	UpdateBBoxSystem,
	TransformShapeSystem,
	UpdateStaticClippingSystem
};
pub use crate::input::{
	UpdateCameraSystem,
	PrintDebugSystem,
	SelectTargetSystem,
	ManipulateSelectedShapeSystem,
	CreateShapeSystem,
	DuplicateShapeSystem,
	DeleteShapeSystem,
	UpdateSelectionBox
};
pub use crate::player::{ShapeTargetingSystem};
pub use crate::collide::{
	PlayerCollisionDetectionSystem,
	PlayerStaticCollisionSystem,
	MovePlayerSystem,
	UpdatePlayerBBox,
};
pub use crate::gravity::{
	PlayerGravitySystem
};
pub use crate::coin::{
	CoinSpinningSystem,
	PlayerCoinCollisionSystem
};
pub use crate::cleanup::{ShapeCleanupSystem};
