use bevy::prelude::*;

use crate::grid::{GridClicked, GridPosition, Tile};
use crate::tile_overlays::{OverlayLayer, TileOverlay, set_overlay_at};

use crate::{
    PlayerTurnState,
    units::{self, PlayerUnit},
};

#[derive(Resource, Default, Deref, DerefMut, Clone, Copy)]
pub struct SelectedPosition(pub Option<GridPosition>);

#[derive(Event)]
pub struct Deselect;

#[derive(Component)]
pub struct ValidMovement;

pub fn grid_clicked(
    mut commands: Commands,
    mut ev_grid_clicked: MessageReader<GridClicked>,
    mut next_state: ResMut<NextState<PlayerTurnState>>,
    mut selected_unit: ResMut<units::SelectedUnit>,
    mut selected_position: ResMut<SelectedPosition>,
    query: Query<(&GridPosition, &units::Movement), With<PlayerUnit>>,

    mut overlays: Query<(Entity, &mut TileOverlay)>,
) {
    for ev in ev_grid_clicked.read() {
        if let Some(entity) = ev.entity {
            commands.trigger(Deselect);

            selected_unit.0 = Some(entity);
            next_state.set(PlayerTurnState::SelectedUnit);
            return;
        }

        if let Some(player_entity) = **selected_unit {
            let (origin, movement) = query.get(player_entity).unwrap();
            let range = movement.range;

            for (_, mut overlay) in overlays.iter_mut() {
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

pub fn on_deselect(_: On<Deselect>, mut commands: Commands, mut overlays: Query<(Entity, &Tile)>) {
    for (entity, mut tile) in overlays.iter_mut() {
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
    query: Query<(&GridPosition, &units::Movement), With<PlayerUnit>>,
    tiles: Query<(Entity, &Children, &Tile)>,
    mut overlays: Query<&mut TileOverlay>,
) {
    let entity = selected_unit.unwrap();
    let (origin, movement) = query.get(entity).unwrap();
    let range = movement.range;

    for (entity, children, tile) in tiles {
        let dx = (tile.x - origin.x).abs();
        let dy = (tile.y - origin.y).abs();

        let child = children.first().unwrap();
        let mut overlay = overlays.get_mut(*child).unwrap();

        if dx == 0 && dy == 0 {
            overlay.selected = true;
        }

        // TODO: Don't show if tile contains a unit
        if range.contains(*origin, tile.into()) {
            commands.entity(entity).insert(ValidMovement);
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
