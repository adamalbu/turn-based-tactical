#![allow(clippy::type_complexity)]

use bevy::{log::LogPlugin, prelude::*};

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

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Tactical Game".into(),
                        name: Some("turn-based-tactical".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(LogPlugin {
                    level: bevy::log::Level::DEBUG,
                    ..Default::default()
                }),
            MeshPickingPlugin,
            player::plugin,
            enemy::plugin,
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
