use bevy::prelude::*;

use crate::{
    PlayerTurnState,
    units::{self, PlayerUnit},
};

pub const TILE_SIZE: f32 = 64.0;
pub const MAP_WIDTH: u32 = 12;
pub const MAP_HEIGHT: u32 = 9;

const THICKNESS: f32 = 2.0;

#[derive(Message, Debug, Clone, Copy)]
pub struct GridClicked {
    pub position: GridPosition,
    pub entity: Option<Entity>,
}

#[derive(Resource, Default, Deref, DerefMut, Clone, Copy)]
pub struct SelectedPosition(pub Option<GridPosition>);

#[derive(Component)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
}

impl From<Tile> for Vec2 {
    fn from(val: Tile) -> Self {
        Vec2 {
            x: val.x as f32,
            y: val.y as f32,
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl From<Tile> for GridPosition {
    fn from(value: Tile) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<&Tile> for GridPosition {
    fn from(value: &Tile) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

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
    fn set_layer(&mut self, layer: OverlayLayer, enabled: bool) {
        match layer {
            OverlayLayer::Range => self.range = enabled,
            OverlayLayer::Selected => self.selected = enabled,
            OverlayLayer::Hover => self.hover = enabled,
        }
    }
    fn get_material(&self, materials: &Res<TileOverlayMaterials>) -> Handle<ColorMaterial> {
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
    none: Handle<ColorMaterial>,
    range: Handle<ColorMaterial>,
    selected: Handle<ColorMaterial>,
    hover: Handle<ColorMaterial>,
}

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let overlay_materials = TileOverlayMaterials {
        none: materials.add(Color::NONE),
        range: materials.add(Color::srgba(0.0, 1.0, 0.0, 0.5)),
        selected: materials.add(Color::srgba(0.0, 0.0, 1.0, 0.5)),
        hover: materials.add(Color::srgba(1.0, 1.0, 0.0, 0.5)),
    };

    let hover_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE));

    let tile_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE).to_ring(THICKNESS));
    let tile_material = materials.add(Color::BLACK);

    let overlay_mesh = meshes.add(Rectangle::new(
        TILE_SIZE - THICKNESS * 2.0,
        TILE_SIZE - THICKNESS * 2.0,
    ));

    let offset = Vec2::new(
        -(MAP_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
        -(MAP_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
    );

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let pos = Vec2::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE) + offset;

            commands
                .spawn((
                    Mesh2d(hover_mesh.clone()),
                    Transform::from_xyz(pos.x, pos.y, 1.0),
                    Tile {
                        x: x as i32,
                        y: y as i32,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh2d(overlay_mesh.clone()),
                        MeshMaterial2d(overlay_materials.none.clone()),
                        TileOverlay::default(),
                        Transform::from_xyz(0.0, 0.0, -0.1),
                    ));

                    parent.spawn((
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(tile_material.clone()),
                        Transform::from_xyz(0.0, 0.0, -0.2),
                    ));
                })
                .observe(update_overlay_material::<Pointer<Over>>(
                    OverlayLayer::Hover,
                    true,
                ))
                .observe(update_overlay_material::<Pointer<Out>>(
                    OverlayLayer::Hover,
                    false,
                ))
                .observe(
                    |event: On<Pointer<Click>>,
                     tiles: Query<&Tile>,
                     players: Query<(Entity, &GridPosition), With<PlayerUnit>>,
                     mut ev_grid_clicked: MessageWriter<GridClicked>| {
                        let clicked_coords: GridPosition = tiles.get(event.entity).unwrap().into();

                        if let Some((player, _)) = players
                            .iter()
                            .find(|(_, position)| **position == clicked_coords)
                        {
                            ev_grid_clicked.write(GridClicked {
                                position: clicked_coords,
                                entity: Some(player),
                            });
                        } else {
                            ev_grid_clicked.write(GridClicked {
                                position: clicked_coords,
                                entity: None,
                            });
                        };
                    },
                );
        }
    }

    commands.insert_resource(overlay_materials);
}

#[allow(clippy::type_complexity)]
fn update_overlay_material<E: EntityEvent>(
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

pub fn grid_clicked(
    mut ev_grid_clicked: MessageReader<GridClicked>,
    mut next_state: ResMut<NextState<PlayerTurnState>>,
    mut selected_unit: ResMut<units::SelectedUnit>,
    mut selected_position: ResMut<SelectedPosition>,
    query: Query<(&GridPosition, &units::Movement), With<PlayerUnit>>,

    mut overlays: Query<&mut TileOverlay>,
) {
    for ev in ev_grid_clicked.read() {
        for mut overlay in overlays.iter_mut() {
            overlay.selected = false;
        }

        if let Some(entity) = ev.entity {
            selected_unit.0 = Some(entity);
            next_state.set(PlayerTurnState::SelectedUnit);
            return;
        }

        if let Some(player_entity) = **selected_unit {
            let (origin, movement) = query.get(player_entity).unwrap();
            let range = movement.range;

            if range.contains(*origin, ev.position) {
                selected_position.0 = Some(ev.position);
                next_state.set(PlayerTurnState::SelectedPosition);
                return;
            }
        }

        selected_unit.0 = None;
        next_state.set(PlayerTurnState::None);
    }
}

pub fn deselect(mut overlays: Query<&mut TileOverlay>) {
    for mut overlay in overlays.iter_mut() {
        overlay.range = false;
        overlay.selected = false;
    }
}

pub fn player_selected(
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

pub fn update_overlay_materials(
    overlays: Query<(&TileOverlay, &mut MeshMaterial2d<ColorMaterial>), Changed<TileOverlay>>,
    materials: Res<TileOverlayMaterials>,
) {
    for (overlay, mut material) in overlays {
        material.0 = overlay.get_material(&materials);
    }
}
