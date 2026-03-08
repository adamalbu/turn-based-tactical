#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use crate::units::{enemy, player};

mod grid;
mod interaction;
mod tile_overlays;
mod ui;
mod units;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    PlayerTurn,
    EnemyTurn,
    Win,
    Lose,
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Tactical Game".into(),
                    name: Some("turn-based-tactical".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            MeshPickingPlugin,
        ))
        .insert_resource(ClearColor(Color::WHITE))
        .init_resource::<units::SelectedUnit>()
        .init_resource::<interaction::SelectedPosition>()
        .init_resource::<units::player::PlayerAssets>()
        .init_resource::<units::enemy::EnemyAssets>()
        .init_resource::<units::HealthBarAssets>()
        .init_state::<GameState>()
        .init_state::<player::TurnState>()
        .init_state::<enemy::TurnState>()
        .add_message::<grid::GridClicked>()
        .add_message::<ui::MoveButtonClicked>()
        .add_observer(interaction::on_deselect)
        .add_systems(
            Startup,
            (spawn_camera, grid::spawn, units::setup.after(grid::spawn)),
        )
        .add_systems(
            OnEnter(player::TurnState::SelectedUnit),
            interaction::selected_player.run_if(in_state(GameState::PlayerTurn)),
        )
        .add_systems(
            OnEnter(player::TurnState::None),
            (interaction::deselect, units::player::check_player_turn_over)
                .run_if(in_state(GameState::PlayerTurn)),
        )
        .add_systems(
            OnEnter(player::TurnState::SelectedPosition),
            (interaction::selected_position, ui::spawn_action_bar),
        )
        .add_systems(
            OnExit(player::TurnState::SelectedPosition),
            ui::despawn_action_bar,
        )
        .add_systems(OnEnter(enemy::TurnState::TakeDamage), enemy::take_damage)
        .add_systems(OnEnter(enemy::TurnState::Move), enemy::r#move)
        .add_systems(
            OnEnter(GameState::PlayerTurn),
            units::player::on_player_turn,
        )
        .add_systems(OnEnter(GameState::EnemyTurn), units::enemy::on_enemy_turn)
        .add_systems(
            Update,
            (
                tile_overlays::update_range_overlay.before(tile_overlays::update_overlay_materials),
                tile_overlays::update_overlay_materials,
                interaction::grid_clicked,
                (ui::handle_move_button, units::move_unit)
                    .run_if(in_state(player::TurnState::SelectedPosition)),
                units::update_positions,
                units::update_health_bar,
            ),
        )
        .add_systems(PostUpdate, units::check_win)
        .run();
}
