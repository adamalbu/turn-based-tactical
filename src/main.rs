use bevy::prelude::*;

mod grid;
mod units;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
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
        .add_message::<units::PlayerSelected>()
        .add_systems(
            Startup,
            (
                spawn_camera,
                grid::spawn,
                units::spawn_player.after(grid::spawn),
            ),
        )
        .add_systems(Update, grid::show_player_move_range)
        .run();
}
