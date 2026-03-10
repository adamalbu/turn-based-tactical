use bevy::prelude::*;

use crate::{
    grid::{GridPosition, Tile},
    units::{SelectedUnit, UnitActionRange},
};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum OverlayLayer {
    Move,
    Attack,
    Selected,
    Hover,
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct TileOverlay {
    r#move: bool,
    pub move_transparent: bool,
    attack: bool,
    pub attack_transparent: bool,
    pub selected: bool,
    pub hover: bool,
}

impl TileOverlay {
    pub fn set_layer(&mut self, layer: OverlayLayer, enabled: bool) {
        match layer {
            OverlayLayer::Move => self.r#move = enabled,
            OverlayLayer::Attack => self.attack = enabled,
            OverlayLayer::Selected => self.selected = enabled,
            OverlayLayer::Hover => self.hover = enabled,
        }
    }

    pub fn get_material(&self, materials: &Res<Materials>) -> Handle<ColorMaterial> {
        if self.hover {
            materials.hover.clone()
        } else if self.selected {
            materials.selected.clone()
        } else if self.r#move && self.attack {
            materials.move_attack.clone()
        } else if self.r#move {
            materials.r#move.clone()
        } else if self.attack {
            materials.attack.clone()
        } else if self.move_transparent && self.attack_transparent {
            materials.move_attack_transparent.clone()
        } else if self.move_transparent {
            materials.move_transparent.clone()
        } else if self.attack_transparent {
            materials.attack_transparent.clone()
        } else {
            materials.none.clone()
        }
    }
}

#[derive(Resource, Clone)]
pub struct Materials {
    pub none: Handle<ColorMaterial>,
    pub r#move: Handle<ColorMaterial>,
    pub move_transparent: Handle<ColorMaterial>,
    pub attack_transparent: Handle<ColorMaterial>,
    pub attack: Handle<ColorMaterial>,
    pub move_attack: Handle<ColorMaterial>,
    pub move_attack_transparent: Handle<ColorMaterial>,
    pub selected: Handle<ColorMaterial>,
    pub hover: Handle<ColorMaterial>,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(PreStartup, setup_assets).add_systems(
        Update,
        (
            update_range_overlay.before(update_overlay_mesh_materials),
            update_overlay_mesh_materials,
        ),
    );
}

pub fn setup_assets(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(Materials {
        none: materials.add(Color::NONE),
        r#move: materials.add(Color::srgba(0.0, 1.0, 0.0, 0.5)),
        move_transparent: materials.add(Color::srgba(0.0, 1.0, 0.0, 0.3)),
        attack: materials.add(Color::srgba(1.0, 0.0, 0.0, 0.5)),
        attack_transparent: materials.add(Color::srgba(1.0, 0.0, 0.0, 0.3)),
        move_attack: materials.add(Color::srgba(0.5, 0.5, 0.0, 0.5)),
        move_attack_transparent: materials.add(Color::srgba(0.5, 0.5, 0.0, 0.3)),
        selected: materials.add(Color::srgba(0.0, 0.0, 1.0, 0.5)),
        hover: materials.add(Color::srgba(1.0, 1.0, 0.0, 0.5)),
    });
}

pub fn update_overlay_material<E: EntityEvent>(
    layer: OverlayLayer,
    enabled: bool,
) -> impl Fn(On<E>, Query<&mut TileOverlay>) {
    move |event, mut overlays| {
        let mut overlay = overlays.get_mut(event.event_target()).unwrap();
        overlay.set_layer(layer, enabled);
    }
}

pub fn set_overlay_at(
    pos: GridPosition,
    layer: OverlayLayer,
    enabled: bool,
    overlay: &mut Query<(&mut Tile, &mut TileOverlay)>,
) {
    for (tile, mut overlay) in overlay {
        if tile.x == pos.x && tile.y == pos.y {
            overlay.set_layer(layer, enabled);
        }
    }
}

pub fn update_range_overlay(
    tiles: Query<(&Tile, &mut TileOverlay)>,
    action_range: ResMut<UnitActionRange>,
    selected_unit: Res<SelectedUnit>,
) {
    if selected_unit.is_changed() || action_range.is_changed() {
        for (tile, mut overlay) in tiles {
            let valid_movement = selected_unit
                .0
                .is_some_and(|unit| action_range.move_tiles[&unit].contains(&tile.into()));

            let valid_attack = selected_unit
                .0
                .is_some_and(|unit| action_range.attack_tiles[&unit].contains(&tile.into()));
            overlay.r#move = valid_movement;
            overlay.attack = valid_attack;
        }
    }
}

pub fn update_overlay_mesh_materials(
    overlays: Query<(&TileOverlay, &Children), Changed<TileOverlay>>,
    mut overlay_mesh_materials: Query<&mut MeshMaterial2d<ColorMaterial>>,
    overlay_materials_handles: Res<Materials>,
) {
    for (overlay, children) in overlays {
        let overlay_child = children.first().unwrap();
        let mut material = overlay_mesh_materials.get_mut(*overlay_child).unwrap();
        material.0 = overlay.get_material(&overlay_materials_handles);
    }
}
