use bevy::prelude::*;

use self::pawn::{setup_asset_pawn, test_setup, spawn_pawn_timer, spawn_pawn_event, PawnSpawn, PawnSetup, pawn_spawn_anim_is_end, pawn_action_anim_is_end};

mod pawn;

pub struct ChessGamePlugin;

impl Plugin for ChessGamePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_state::<PawnSetup>()
        .add_event::<PawnSpawn>()
        .add_systems(OnEnter(PawnSetup::Befor), (
            setup_asset_pawn,
        ))
        .add_systems(OnEnter(PawnSetup::After), (
            test_setup,
        ))
        .add_systems(Update, (
            spawn_pawn_timer,
            spawn_pawn_event,
            pawn_spawn_anim_is_end,
            pawn_action_anim_is_end,
        ));
    }
}