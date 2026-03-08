use bevy::prelude::*;

use crate::grid::{self, GridClicked, GridPosition, Tile, los};
use crate::tile_overlays::{OverlayLayer, TileOverlay, set_overlay_at};

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
    mut tiles: Query<(&mut Tile, &mut TileOverlay, Has<ValidMovement>)>,
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
            for (_, mut overlay, _) in tiles.iter_mut() {
                overlay.selected = false;
            }

            if tiles.get(ev.tile).unwrap().2 {
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
    tiles: Query<Entity, With<Tile>>,
    mut overlays: Query<&mut TileOverlay>,
) {
    for entity in tiles.iter() {
        let mut overlay = overlays.get_mut(entity).unwrap();
        overlay.selected = false;
        overlay.attack = false;
        commands.entity(entity).remove::<ValidMovement>();
    }
}

pub fn deselect(mut commands: Commands, mut selected_unit: ResMut<units::SelectedUnit>) {
    selected_unit.0 = None;
    commands.trigger(Deselect);
}

#[allow(clippy::too_many_arguments)]
pub fn selected_player(
    mut commands: Commands,
    selected_unit: ResMut<units::SelectedUnit>,
    player: Query<(&GridPosition, &units::Movement), With<PlayerUnit>>,
    player_attack: Query<&Attack, With<PlayerUnit>>,
    unit_positions: Query<&GridPosition, With<Unit>>,
    tiles: Query<(Entity, &Tile)>,
    only_tiles: Query<&Tile>,
    mut overlays: Query<&mut TileOverlay>,
    tilemap: Res<grid::Tilemap>,
) {
    let entity = selected_unit.unwrap();
    let (origin, movement) = player.get(entity).unwrap();
    let attack_range = player_attack.get(entity).unwrap().range;
    let move_range = movement.range;

    for (tile_entity, tile) in tiles {
        let dx = (tile.x - origin.x).abs();
        let dy = (tile.y - origin.y).abs();

        let mut overlay = overlays.get_mut(tile_entity).unwrap();

        if dx == 0 && dy == 0 {
            overlay.selected = true;
        }

        if attack_range.contains(*origin, GridPosition::from(*tile)) {
            overlay.attack = true;
        }

        if move_range.contains_with_los(*origin, GridPosition::from(*tile), |from, to| {
            los(from, to, &tilemap, &only_tiles)
        }) {
            let over_unit = unit_positions
                .iter()
                .any(|t| t == &GridPosition::from(*tile));

            if !over_unit {
                commands.entity(tile_entity).insert(ValidMovement);
            }
        }
    }
}

pub fn selected_position(
    selected_position: Res<SelectedPosition>,
    mut tiles: Query<(&mut Tile, &mut TileOverlay)>,
) {
    set_overlay_at(
        selected_position.unwrap(),
        OverlayLayer::Selected,
        true,
        &mut tiles,
    );
}
