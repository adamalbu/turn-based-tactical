use bevy::prelude::*;

mod grid;
mod interaction;
mod tile_overlays;
mod ui;
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
        .init_resource::<units::SelectedUnit>()
        .init_resource::<interaction::SelectedPosition>()
        .init_state::<PlayerTurnState>()
        .add_message::<grid::GridClicked>()
        .add_message::<ui::MoveButtonClicked>()
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
            interaction::player_selected,
        )
        .add_systems(OnEnter(PlayerTurnState::None), interaction::deselect)
        .add_systems(
            OnEnter(PlayerTurnState::SelectedPosition),
            (interaction::selected_position, ui::spawn_action_bar),
        )
        .add_systems(
            OnExit(PlayerTurnState::SelectedPosition),
            ui::despawn_action_bar,
        )
        .add_systems(
            Update,
            (
                tile_overlays::update_overlay_materials,
                interaction::grid_clicked,
                ui::handle_move_button.run_if(in_state(PlayerTurnState::SelectedPosition)),
                units::move_unit.run_if(in_state(PlayerTurnState::SelectedPosition)),
                units::update_positions,
            ),
        )
        .run();
}
