use bevy::prelude::*;

use self::{pawn::{setup_asset_pawn, test_setup, spawn_pawn_timer, spawn_pawn_event, PawnSpawn, PawnSetup, pawn_spawn_anim_is_end, pawn_action_anim_is_end, selected_event, pawn_combination_is_end, OtherSpawn}, bishop::{setup_asset_bishop, bishop_spawn_event, bishop_spawn_anim_is_end, cancel_path, bishop_attacking}};

mod pawn;
mod bishop;

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
        .add_event::<OtherSpawn>()
        .add_systems(OnEnter(PawnSetup::Befor),(
            setup_asset_bishop,
            setup_asset_pawn,
        ).chain().in_set(ScadulSet::SetUp))
        .add_systems(OnEnter(PawnSetup::After),(
            test_setup,
        ).before(ScadulSet::SetUp))
        .add_systems(Update, (
            (
                cancel_path,
            ).before(ScadulSet::Spawn),
            (
                spawn_pawn_timer,
                spawn_pawn_event,
                pawn_spawn_anim_is_end,
                pawn_action_anim_is_end,
                bishop_spawn_event,
                bishop_spawn_anim_is_end,
                selected_event
            ).in_set(ScadulSet::Spawn),
            (
                pawn_combination_is_end,
                bishop_attacking
            ).before(ScadulSet::Spawn)
        ));
    }
}