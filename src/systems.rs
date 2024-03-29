mod shape_systems;
//for easy import of all systems
pub use self::shape_systems::*;
pub use crate::cleanup::ShapeCleanupSystem;
pub use crate::coin::{CoinSpinningSystem, PlayerCoinCollisionSystem};
pub use crate::collide::systems::*;
pub use crate::draw::systems::*;
pub use crate::gravity::PlayerGravitySystem;
pub use crate::input::systems::*;
pub use crate::player::ShapeTargetingSystem;
