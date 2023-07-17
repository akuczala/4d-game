use crate::ecs_utils::{Componentable, ModSystem, SystemName};
use crate::systems::*;
use crate::vector::{MatrixTrait, VectorTrait};
use specs::prelude::*;
use std::marker::PhantomData;

// TODO: write in terms of larger blocks of systems
// TODO: make system labels part of the system struct somehow
pub fn get_engine_dispatcher_builder<'a, 'b, V>() -> DispatcherBuilder<'a, 'b>
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    let builder = add_draw_steps::<V>(DispatcherBuilder::new());
    let builder = add_game_steps::<V>(builder);
    builder
}

//start drawing phase. this is first so that we can do world.maintain() before we draw
//for each shape, update clipping boundaries and face visibility
fn add_draw_steps<'a, 'b, V>(builder: DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b>
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    let ph = PhantomData::<V>;
    builder
        .with(VisibilitySystem(ph), VisibilitySystem::NAME, &[])
        //determine what shapes are in front of other shapes
        .with(
            InFrontSystem(ph),
            InFrontSystem::NAME,
            &[VisibilitySystem::NAME],
        )
        //calculate and clip lines for each shape
        .with(
            CalcShapesLinesSystem(ph),
            CalcShapesLinesSystem::NAME,
            &[InFrontSystem::NAME],
        )
        //draw selection box in the space
        .with(
            DrawLineCollectionSystem(ph),
            DrawLineCollectionSystem::NAME,
            &[InFrontSystem::NAME],
        )
        //project lines
        .with(
            TransformDrawLinesSystem(ph),
            TransformDrawLinesSystem::NAME,
            &[CalcShapesLinesSystem::NAME, DrawLineCollectionSystem::NAME],
        )
        // draw the cursor on the d - 1 screen
        .with(
            DrawCursorSystem(ph),
            DrawCursorSystem::NAME,
            &[TransformDrawLinesSystem::NAME],
        )
}

//start game update phase
fn add_game_steps<'a, 'b, V>(builder: DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b>
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    let ph = PhantomData::<V>;
    builder
        .with(
            UpdateCameraSystem(ph),
            "update_camera",
            &[CalcShapesLinesSystem::NAME],
        )
        .with(
            PlayerGravitySystem(ph),
            "player_gravity",
            &["update_camera"],
        )
        .with(
            PlayerCollisionDetectionSystem(ph),
            "player_collision_detect",
            &["update_camera", "player_gravity"],
        )
        .with(
            PlayerStaticCollisionSystem(ph),
            "player_static_collision",
            &["player_collision_detect"],
        )
        .with(
            PlayerCoinCollisionSystem(ph),
            "player_coin_collision",
            &["player_collision_detect", "player_static_collision"],
        )
        .with(
            MovePlayerSystem(ph),
            "move_player",
            &["player_static_collision", "player_coin_collision"],
        )
        .with(
            ShapeTargetingSystem(ph),
            "shape_targeting",
            &["move_player"],
        )
        .with(
            SelectTargetSystem(ph),
            "select_target",
            &["shape_targeting"],
        )
        .with(
            ManipulateSelectedShapeSystem(ph),
            "manipulate_selected",
            &["select_target"],
        )
        .with(CreateShapeSystem(ph), "create_shape", &[])
        .with(DuplicateShapeSystem(ph), "duplicate_shape", &[])
        .with(DeleteShapeSystem(ph), "delete_shape", &[])
        .with(CoinSpinningSystem(ph), "coin_spinning", &[])
        .with(
            TransformShapeSystem(ModSystem::typed_default(ph)),
            "transform_shapes",
            &["manipulate_selected", "coin_spinning"],
        )
        .with(
            UpdateSelectionBox(ModSystem::typed_default(ph)),
            "update_selection_box",
            &["transform_shapes"],
        )
        .with(UpdatePlayerBBox(ph), "update_player_bbox", &["move_player"]) //merge with above
        .with(
            UpdateBBoxSystem(ModSystem::typed_default(ph)),
            "update_all_bbox",
            &["transform_shapes"],
        ) //if we had moving objects other than player
        .with(
            BBoxHashingSystem(ModSystem::typed_default(ph)),
            BBoxHashingSystem::NAME,
            &["update_all_bbox"],
        )
        .with(
            UpdateBBallSystem(ModSystem::typed_default(ph)),
            "update_all_bball",
            &["transform_shapes"],
        )
        // .with(
        //     UpdateStaticClippingSystem(ModSystem::typed_default(ph)),
        //     "update_static_clipping",
        //     &["transform_shapes"],
        // )
        .with(
            ShapeCleanupSystem(ph),
            "shape_cleanup",
            &["delete_shape", BBoxHashingSystem::NAME],
        )
    //.with(PrintDebugSystem(ph), "print_debug", &["update_camera"])
}
