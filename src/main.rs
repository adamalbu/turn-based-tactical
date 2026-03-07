use bevy::prelude::*;

mod grid;
mod units;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerTurnState {
    #[default]
    None,
    SelectedUnit,
    SelectedPosition,
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
        .insert_resource(units::SelectedUnit::default())
        .insert_resource(grid::SelectedPosition::default())
        .insert_state(PlayerTurnState::default())
        .add_message::<grid::GridClicked>()
        .add_systems(
            Startup,
            (
                spawn_camera,
                grid::spawn,
                units::spawn_player.after(grid::spawn),
            ),
        )
        .add_systems(
            OnEnter(PlayerTurnState::SelectedUnit),
            grid::player_selected,
        )
        .add_systems(OnEnter(PlayerTurnState::None), grid::deselect)
        .add_systems(
            OnEnter(PlayerTurnState::SelectedPosition),
            grid::selected_position,
        )
        .add_systems(Update, (grid::update_overlay_materials, grid::grid_clicked))
        .run();
}
