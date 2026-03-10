use bevy::prelude::*;

use crate::grid::{GridClicked, GridPosition, Tile};
use crate::tile_overlays::{OverlayLayer, TileOverlay, set_overlay_at};

use crate::units::{self, player::PlayerUnit};
use crate::units::{UnitActionRange, player};

#[derive(Resource, Default, Deref, DerefMut, Clone, Copy)]
pub struct SelectedPosition(pub Option<GridPosition>);

#[derive(Event)]
pub struct Deselect;

pub fn grid_clicked(
    mut commands: Commands,
    mut ev_grid_clicked: MessageReader<GridClicked>,
    mut next_state: ResMut<NextState<player::TurnState>>,
    mut selected_unit: ResMut<units::SelectedUnit>,
    mut selected_position: ResMut<SelectedPosition>,
    mut tiles: Query<(&mut Tile, &mut TileOverlay)>,
    action_range: ResMut<UnitActionRange>,
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
            for (_, mut overlay) in tiles.iter_mut() {
                overlay.selected = false;
            }

            let pos = &GridPosition::from(
                *(tiles
                    .transmute_lens::<&mut Tile>()
                    .query()
                    .get_mut(ev.tile)
                    .unwrap()),
            );

            let is_move_tile = action_range
                .move_tiles
                .get(&selected_unit.unwrap())
                .unwrap()
                .contains(pos);

            if is_move_tile {
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
    tiles: Query<Entity, With<Tile>>,
    mut overlays: Query<&mut TileOverlay>,
) {
    for entity in tiles.iter() {
        let mut overlay = overlays.get_mut(entity).unwrap();
        overlay.selected = false;
    }
}

pub fn deselect(mut commands: Commands, mut selected_unit: ResMut<units::SelectedUnit>) {
    selected_unit.0 = None;
    commands.trigger(Deselect);
}

pub fn selected_player(
    selected_unit: ResMut<units::SelectedUnit>,
    player: Query<&GridPosition, With<PlayerUnit>>,
    tiles: Query<(Entity, &Tile)>,
    mut overlays: Query<&mut TileOverlay>,
) {
    let entity = selected_unit.unwrap();
    let origin = player.get(entity).unwrap();

    for (tile_entity, tile) in tiles {
        let dx = (tile.x - origin.x).abs();
        let dy = (tile.y - origin.y).abs();

        let mut overlay = overlays.get_mut(tile_entity).unwrap();

        if dx == 0 && dy == 0 {
            overlay.selected = true;
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
