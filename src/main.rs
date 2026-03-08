#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use crate::units::{enemy, player};

mod grid;
mod interaction;
mod tile_overlays;
mod ui;
mod units;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    PlayerTurn,
    EnemyTurn,
    Win,
    Lose,
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn check_win(
    enemies: Query<&enemy::EnemyUnit>,
    players: Query<&player::PlayerUnit>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if players.count() == 0 {
        next_state.set(GameState::Lose);
    }
    if enemies.count() == 0 {
        next_state.set(GameState::Win);
    }
}

fn player_plugin(app: &mut App) {
    app.init_resource::<units::player::PlayerAssets>()
        .init_state::<player::TurnState>()
        .add_message::<ui::MoveButtonClicked>()
        .init_resource::<interaction::SelectedPosition>()
        .add_observer(interaction::on_deselect)
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
        .add_systems(OnEnter(player::TurnState::End), player::end_turn)
        .add_systems(
            OnEnter(GameState::PlayerTurn),
            units::player::on_player_turn,
        )
        .add_systems(
            Update,
            ui::handle_move_button.run_if(in_state(player::TurnState::SelectedPosition)),
        );
}

fn enemy_plugin(app: &mut App) {
    app.init_resource::<units::enemy::EnemyAssets>()
        .init_state::<enemy::TurnState>()
        .add_systems(OnEnter(enemy::TurnState::TakeDamage), enemy::take_damage)
        .add_systems(OnEnter(enemy::TurnState::Move), enemy::r#move)
        .add_systems(OnEnter(enemy::TurnState::End), enemy::end_turn)
        .add_systems(OnEnter(GameState::EnemyTurn), units::enemy::on_enemy_turn);
}

fn tile_overlays_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            tile_overlays::update_range_overlay.before(tile_overlays::update_overlay_materials),
            tile_overlays::update_overlay_materials,
        ),
    );
}

fn units_plugin(app: &mut App) {
    app.init_resource::<units::SelectedUnit>()
        .init_resource::<units::HealthBarAssets>()
        .add_systems(Startup, units::setup.after(grid::spawn))
        .add_systems(
            Update,
            (
                units::update_positions,
                units::update_health_bar,
                units::move_unit,
            ),
        );
}

fn grid_plugin(app: &mut App) {
    app.add_systems(Startup, grid::spawn)
        .add_message::<grid::GridClicked>()
        .add_systems(Update, interaction::grid_clicked);
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
            player_plugin,
            enemy_plugin,
            tile_overlays_plugin,
            units_plugin,
            grid_plugin,
        ))
        .insert_resource(ClearColor(Color::WHITE))
        .init_state::<GameState>()
        .add_systems(Startup, spawn_camera)
        .add_systems(PostUpdate, check_win)
        .run();
}
