#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use crate::units::{enemy, player};

mod game;
mod grid;
mod interaction;
mod tile_overlays;
mod ui;
mod units;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn enemy_plugin(app: &mut App) {
    app.init_resource::<units::enemy::EnemyAssets>()
        .init_state::<enemy::TurnState>()
        .add_systems(OnEnter(enemy::TurnState::TakeDamage), enemy::take_damage)
        .add_systems(OnEnter(enemy::TurnState::Move), enemy::r#move)
        .add_systems(OnEnter(enemy::TurnState::End), enemy::end_turn)
        .add_systems(
            OnEnter(game::GameState::EnemyTurn),
            units::enemy::on_enemy_turn,
        );
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
            player::plugin,
            enemy_plugin,
            tile_overlays::plugin,
            units::plugin,
            grid::plugin,
            game::plugin,
            ui::plugin,
        ))
        .insert_resource(ClearColor(Color::WHITE))
        .init_state::<game::GameState>()
        .add_systems(Startup, spawn_camera)
        .run();
}
