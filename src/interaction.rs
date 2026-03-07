use bevy::prelude::*;

use crate::grid::{GridClicked, GridPosition, Tile};
use crate::tile_overlays::{OverlayLayer, TileOverlay, set_overlay_at};

use crate::{
    PlayerTurnState,
    units::{self, PlayerUnit},
};

#[derive(Resource, Default, Deref, DerefMut, Clone, Copy)]
pub struct SelectedPosition(pub Option<GridPosition>);

pub fn grid_clicked(
    mut ev_grid_clicked: MessageReader<GridClicked>,
    mut next_state: ResMut<NextState<PlayerTurnState>>,
    mut selected_unit: ResMut<units::SelectedUnit>,
    mut selected_position: ResMut<SelectedPosition>,
    query: Query<(&GridPosition, &units::Movement), With<PlayerUnit>>,

    mut overlays: Query<&mut TileOverlay>,
) {
    for ev in ev_grid_clicked.read() {
        if let Some(entity) = ev.entity {
            deselect(overlays, &mut selected_unit);
            selected_unit.0 = Some(entity);
            next_state.set(PlayerTurnState::SelectedUnit);
            return;
        }

        if let Some(player_entity) = **selected_unit {
            let (origin, movement) = query.get(player_entity).unwrap();
            let range = movement.range;

            for mut overlay in overlays.iter_mut() {
                overlay.selected = false;
            }

            if range.contains(*origin, ev.position) {
                selected_position.0 = Some(ev.position);
                next_state.set(PlayerTurnState::SelectedPosition);
                return;
            }
        }

        next_state.set(PlayerTurnState::None);
    }
}

pub fn deselect(
    mut overlays: Query<&mut TileOverlay>,
    selected_unit: &mut ResMut<units::SelectedUnit>,
) {
    selected_unit.0 = None;
    for mut overlay in overlays.iter_mut() {
        overlay.range = false;
        overlay.selected = false;
    }
}

pub fn deselect_system(
    overlays: Query<&mut TileOverlay>,
    mut selected_unit: ResMut<units::SelectedUnit>,
) {
    deselect(overlays, &mut selected_unit)
}

pub fn selected_player(
    selected_unit: ResMut<units::SelectedUnit>,
    query: Query<(&GridPosition, &units::Movement), With<PlayerUnit>>,
    tiles: Query<(&Children, &Tile)>,
    mut overlays: Query<&mut TileOverlay>,
) {
    let entity = selected_unit.unwrap();
    let (origin, movement) = query.get(entity).unwrap();
    let range = movement.range;

    for (children, tile) in tiles {
        let dx = (tile.x - origin.x).abs();
        let dy = (tile.y - origin.y).abs();

        let child = children.first().unwrap();
        let mut overlay = overlays.get_mut(*child).unwrap();

        if dx == 0 && dy == 0 {
            overlay.selected = true;
        }

        if range.contains(*origin, tile.into()) {
            overlay.set_layer(OverlayLayer::Range, true);
        }
    }
}

pub fn selected_position(
    selected_position: Res<SelectedPosition>,
    tiles: Query<(&Tile, &Children)>,
    mut overlays: Query<&mut TileOverlay>,
) {
    set_overlay_at(
        selected_position.unwrap(),
        OverlayLayer::Selected,
        true,
        &tiles,
        &mut overlays,
    );
}
