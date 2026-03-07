use bevy::prelude::*;

use crate::{
    grid::{GridPosition, Tile},
    interaction::ValidMovement,
};

#[derive(Clone, Copy)]
pub enum OverlayLayer {
    Move,
    Attack,
    Selected,
    Hover,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TileOverlay {
    r#move: bool,
    pub attack: bool,
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

    pub fn get_material(&self, materials: &Res<TileOverlayMaterials>) -> Handle<ColorMaterial> {
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
        } else {
            materials.none.clone()
        }
    }
}

#[derive(Resource, Clone)]
pub struct TileOverlayMaterials {
    pub none: Handle<ColorMaterial>,
    pub r#move: Handle<ColorMaterial>,
    pub attack: Handle<ColorMaterial>,
    pub move_attack: Handle<ColorMaterial>,
    pub selected: Handle<ColorMaterial>,
    pub hover: Handle<ColorMaterial>,
}

pub fn update_overlay_material<E: EntityEvent>(
    layer: OverlayLayer,
    enabled: bool,
) -> impl Fn(On<E>, Query<&mut Tile>) {
    move |event, mut tiles| {
        let mut tile = tiles.get_mut(event.event_target()).unwrap();
        tile.overlay.set_layer(layer, enabled);
    }
}

pub fn set_overlay_at(
    pos: GridPosition,
    layer: OverlayLayer,
    enabled: bool,
    tiles: &mut Query<&mut Tile>,
) {
    for mut tile in tiles {
        if tile.x == pos.x && tile.y == pos.y {
            tile.overlay.set_layer(layer, enabled);
        }
    }
}

pub fn update_range_overlay(tiles: Query<(&mut Tile, Has<ValidMovement>)>) {
    for (mut tile, valid_movement) in tiles {
        tile.overlay.r#move = valid_movement;
    }
}

pub fn update_overlay_materials(
    tiles: Query<(&Tile, &Children), Changed<Tile>>,
    mut overlays: Query<&mut MeshMaterial2d<ColorMaterial>>,
    overlay_materials: Res<TileOverlayMaterials>,
) {
    for (tile, children) in tiles {
        let overlay_child = children.first().unwrap();
        let mut material = overlays.get_mut(*overlay_child).unwrap();
        material.0 = tile.overlay.get_material(&overlay_materials);
    }
}
