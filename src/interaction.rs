use bevy::prelude::*;

use crate::grid::{GridClicked, GridPosition, Tile};
use crate::tile_overlays::{OverlayLayer, set_overlay_at};

use crate::units::{self, player::PlayerUnit};
use crate::units::{Attack, Unit, player};

#[derive(Resource, Default, Deref, DerefMut, Clone, Copy)]
pub struct SelectedPosition(pub Option<GridPosition>);

#[derive(Event)]
pub struct Deselect;

#[derive(Component)]
pub struct ValidMovement;

pub fn grid_clicked(
    mut commands: Commands,
    mut ev_grid_clicked: MessageReader<GridClicked>,
    mut next_state: ResMut<NextState<player::TurnState>>,
    mut selected_unit: ResMut<units::SelectedUnit>,
    mut selected_position: ResMut<SelectedPosition>,
    mut tiles: Query<(&mut Tile, Has<ValidMovement>)>,
) {
    for ev in ev_grid_clicked.read() {
        // Select unit
        if let Some(entity) = ev.unit {
            commands.trigger(Deselect);

            selected_unit.0 = Some(entity);
            next_state.set(player::TurnState::SelectedUnit);
            return;
        }

        // Select position
        if (**selected_unit).is_some() {
            // Clear selection
            for (mut tile, _) in tiles.iter_mut() {
                tile.overlay.selected = false;
            }

            if tiles.get(ev.tile).unwrap().1 {
                selected_position.0 = Some(ev.click_pos);
                next_state.set(player::TurnState::SelectedPosition);
                return;
            }
        }

        next_state.set(player::TurnState::None);
    }
}

pub fn on_deselect(
    _: On<Deselect>,
    mut commands: Commands,
    mut overlays: Query<(Entity, &mut Tile)>,
) {
    for (entity, mut tile) in overlays.iter_mut() {
        tile.overlay.selected = false;
        tile.overlay.attack = false;
        commands.entity(entity).remove::<ValidMovement>();
    }
}

pub fn deselect(mut commands: Commands, mut selected_unit: ResMut<units::SelectedUnit>) {
    selected_unit.0 = None;
    commands.trigger(Deselect);
}

pub fn selected_player(
    mut commands: Commands,
    selected_unit: ResMut<units::SelectedUnit>,
    player: Query<(&GridPosition, &units::Movement), With<PlayerUnit>>,
    player_attack: Query<&Attack, With<PlayerUnit>>,
    unit_positions: Query<&GridPosition, With<Unit>>,
    tiles: Query<(Entity, &mut Tile)>,
) {
    let entity = selected_unit.unwrap();
    let (origin, movement) = player.get(entity).unwrap();
    let attack_range = player_attack.get(entity).unwrap().range;
    let move_range = movement.range;

    for (tile_entity, mut tile) in tiles {
        let dx = (tile.x - origin.x).abs();
        let dy = (tile.y - origin.y).abs();

        if dx == 0 && dy == 0 {
            tile.overlay.selected = true;
        }

        if attack_range.contains(*origin, GridPosition::from(*tile)) {
            tile.overlay.attack = true;
        }

        if move_range.contains(*origin, GridPosition::from(*tile)) {
            let over_unit = unit_positions
                .iter()
                .any(|t| t == &GridPosition::from(*tile));

            if !over_unit {
                commands.entity(tile_entity).insert(ValidMovement);
            }
        }
    }
}

pub fn selected_position(selected_position: Res<SelectedPosition>, mut tiles: Query<&mut Tile>) {
    set_overlay_at(
        selected_position.unwrap(),
        OverlayLayer::Selected,
        true,
        &mut tiles,
    );
}
