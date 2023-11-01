use bevy::prelude::*;

use self::pawn::{setup_asset_pawn, test_setup, spawn_pawn_timer, spawn_pawn_event, PawnSpawn, PawnSetup, pawn_spawn_anim_is_end, pawn_action_anim_is_end, selected_event, pawn_combination_is_end};

mod pawn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
enum ScadulSet {
    Spawn, //insert가 들어가거나 하는 형태의 시스템들
    SetUp  //리소스가 생성되는 형태의 시스템들
}

pub struct ChessGamePlugin;

impl Plugin for ChessGamePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_state::<PawnSetup>()
        .add_event::<PawnSpawn>()
        .add_systems(OnEnter(PawnSetup::Befor),(
            setup_asset_pawn,
        ).in_set(ScadulSet::SetUp))
        .add_systems(OnEnter(PawnSetup::After),(
            test_setup,
        ).before(ScadulSet::SetUp))
        .add_systems(Update, (
            spawn_pawn_timer,
            spawn_pawn_event,
            pawn_spawn_anim_is_end,
            pawn_action_anim_is_end,
            selected_event
        ).in_set(ScadulSet::Spawn))
        .add_systems(Update, (
            pawn_combination_is_end,
        ).before(ScadulSet::Spawn));
    }
}