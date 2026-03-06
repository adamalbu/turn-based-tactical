use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tactical Game".into(),
                name: Some("turn-based-tactical".into()),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .run();
}
