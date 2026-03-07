use bevy::prelude::*;

use crate::grid::{GridPosition, Tile};

#[derive(Clone, Copy)]
pub enum OverlayLayer {
    Range,
    Selected,
    Hover,
}

#[derive(Component, Default, Clone, Copy)]
pub struct TileOverlay {
    pub range: bool,
    pub selected: bool,
    pub hover: bool,
}

impl TileOverlay {
    pub fn set_layer(&mut self, layer: OverlayLayer, enabled: bool) {
        match layer {
            OverlayLayer::Range => self.range = enabled,
            OverlayLayer::Selected => self.selected = enabled,
            OverlayLayer::Hover => self.hover = enabled,
        }
    }

    pub fn get_material(&self, materials: &Res<TileOverlayMaterials>) -> Handle<ColorMaterial> {
        if self.hover {
            materials.hover.clone()
        } else if self.selected {
            materials.selected.clone()
        } else if self.range {
            materials.range.clone()
        } else {
            materials.none.clone()
        }
    }
}

#[derive(Resource, Clone)]
pub struct TileOverlayMaterials {
    pub none: Handle<ColorMaterial>,
    pub range: Handle<ColorMaterial>,
    pub selected: Handle<ColorMaterial>,
    pub hover: Handle<ColorMaterial>,
}

pub fn update_overlay_material<E: EntityEvent>(
    layer: OverlayLayer,
    enabled: bool,
) -> impl Fn(On<E>, Query<&Children, With<Tile>>, Query<&mut TileOverlay>) {
    move |event, query, mut overlays| {
        let children = query.get(event.event_target()).unwrap();
        let child = children.first().unwrap();

        let mut overlay = overlays.get_mut(*child).unwrap();
        overlay.set_layer(layer, enabled);
    }
}

pub fn set_overlay_at(
    pos: GridPosition,
    layer: OverlayLayer,
    enabled: bool,
    tiles: &Query<(&Tile, &Children)>,
    overlays: &mut Query<&mut TileOverlay>,
) {
    for (tile, children) in tiles {
        if tile.x == pos.x && tile.y == pos.y {
            let child = children.first().unwrap();
            if let Ok(mut overlay) = overlays.get_mut(*child) {
                overlay.set_layer(layer, enabled);
            }
            return;
        }
    }
}

pub fn update_overlay_materials(
    overlays: Query<(&TileOverlay, &mut MeshMaterial2d<ColorMaterial>), Changed<TileOverlay>>,
    materials: Res<TileOverlayMaterials>,
) {
    for (overlay, mut material) in overlays {
        material.0 = overlay.get_material(&materials);
    }
}
