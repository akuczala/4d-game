use std::marker::PhantomData;
use specs::prelude::*;
use crate::systems::*;
use crate::vector::{VectorTrait};
use crate::draw::DrawSelectionBox;

// todo: write in terms of larger blocks of systems
pub fn get_engine_dispatcher_builder<'a, 'b, V: VectorTrait>() -> DispatcherBuilder<'a,'b> {
    let builder = add_draw_steps::<V>(DispatcherBuilder::new());
    let builder = add_game_steps::<V>(builder);
    builder
}

//start drawing phase. this is first so that we can do world.maintain() before we draw
//for each shape, update clipping boundaries and face visibility
fn add_draw_steps<'a, 'b, V: VectorTrait>(builder: DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b> {
      let ph = PhantomData::<V>;
      builder
      .with(VisibilitySystem(ph),"visibility",
              &[])
        //determine what shapes are in front of other shapes
        .with(InFrontSystem(ph),"in_front",
              &["visibility"])
        //calculate and clip lines for each shape
        .with(CalcShapesLinesSystem(ph),"calc_shapes_lines",
              &["in_front"])
        //draw selection box in the space
        .with(DrawSelectionBox(ph),"draw_selection_box",
              &["in_front"])
        //project lines
        .with(TransformDrawLinesSystem(ph),"transform_draw_lines",
              &["calc_shapes_lines","draw_selection_box"])
        // draw the cursor on the d - 1 screen
        .with(DrawCursorSystem(ph),"draw_cursor",
              &["calc_shapes_lines","draw_selection_box"])
}

//start game update phase
fn add_game_steps<'a, 'b, V: VectorTrait>(builder: DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b> {
      let ph = PhantomData::<V>;
      builder
      .with(UpdateCameraSystem(ph),"update_camera",
              &["calc_shapes_lines"])
        .with(PlayerGravitySystem(ph), "player_gravity",
              &["update_camera"])
        .with(PlayerCollisionDetectionSystem(ph),"player_collision_detect",
              &["update_camera","player_gravity"])
        .with(PlayerStaticCollisionSystem(ph),"player_static_collision",
              &["player_collision_detect"])
        .with(PlayerCoinCollisionSystem(ph),"player_coin_collision",
              &["player_collision_detect","player_static_collision"])
        .with(MovePlayerSystem(ph),"move_player",
              &["player_static_collision","player_coin_collision"])
        .with(ShapeTargetingSystem(ph),"shape_targeting",
              &["move_player"])
        .with(SelectTargetSystem(ph),"select_target",
              &["shape_targeting"])
        .with(ManipulateSelectedShapeSystem(ph), "manipulate_selected",
              &["select_target"])
        .with(UpdatePlayerBBox(ph),"update_player_bbox",
              &["move_player"]) //merge with above
        .with(
            UpdateBBoxSystem{
                  ph,
                  modified: Default::default(),
                  reader_id: Default::default()
            },
            "update_all_bbox",
            &["manipulate_selected"]) //if we had moving objects other than player
        .with(
            UpdateBBallSystem{
                  ph,
                  modified: Default::default(),
                  reader_id: Default::default()
            },
             "update_all_bball",
              &["manipulate_selected"]
            )
      //   .with(CoinSpinningSystem(ph),"coin_spinning",
      //         &[])
        .with(ShapeCleanupSystem(ph),"shape_cleanup",
              &["player_coin_collision"])
        .with(PrintDebugSystem(ph),"print_debug", &["update_camera"])
}