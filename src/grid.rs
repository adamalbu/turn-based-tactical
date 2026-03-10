use std::{collections::HashMap, ops::Sub};

use bevy::prelude::*;

use crate::{
    game, interaction,
    tile_overlays::{self, OverlayLayer, TileOverlay, update_overlay_material},
    units::{
        Unit, UnitActionRange,
        player::{HasMoved, PlayerUnit},
    },
};

pub const TILE_SIZE: f32 = 64.0;

const THICKNESS: f32 = 2.0;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Tilemap(pub HashMap<GridPosition, Entity>);

#[derive(Message, Debug, Clone, Copy)]
pub struct GridClicked {
    pub tile: Entity,
    pub click_pos: GridPosition,
    pub unit: Option<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Floor,
    Wall,
}

#[derive(Resource, Default)]
pub struct TileAssets {
    floor_mesh: Handle<Mesh>,
    floor_material: Handle<ColorMaterial>,
    wall_mesh: Handle<Mesh>,
    wall_material: Handle<ColorMaterial>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub r#type: TileType,
}

impl Tile {
    pub fn new(x: i32, y: i32, r#type: TileType) -> Self {
        Self { x, y, r#type }
    }
}

impl From<Tile> for Vec2 {
    fn from(val: Tile) -> Self {
        Vec2 {
            x: val.x as f32,
            y: val.y as f32,
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl Sub for GridPosition {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: (self.x - rhs.x),
            y: (self.y - rhs.y),
        }
    }
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

pub fn plugin(app: &mut App) {
    app.add_systems(PreStartup, setup_assets)
        .add_systems(Startup, (spawn, build_tilemap.after(spawn)))
        .add_message::<GridClicked>()
        .add_systems(Update, interaction::grid_clicked);
}

pub fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(TileAssets {
        floor_mesh: meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE).to_ring(THICKNESS)),
        floor_material: materials.add(Color::BLACK),
        wall_mesh: meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE)),
        wall_material: materials.add(Color::srgb(0.7, 0.0, 0.0)),
    });
}

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    tile_assets: ResMut<TileAssets>,
    overlay_materials: Res<tile_overlays::Materials>,
) {
    let hover_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE));

    let overlay_mesh = meshes.add(Rectangle::new(
        TILE_SIZE - THICKNESS * 2.0,
        TILE_SIZE - THICKNESS * 2.0,
    ));

    let map_width = game::LEVEL[0].len();
    let map_height = game::LEVEL.len();

    let offset = Vec2::new(
        -(map_width as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
        -(map_height as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
    );

    for (y, row) in game::LEVEL.iter().enumerate().rev() {
        for (x, char) in row.chars().enumerate() {
            let pos = Vec2::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE) + offset;

            let (tile_mesh, tile_material, tile_type) = match char {
                'W' => (
                    tile_assets.wall_mesh.clone(),
                    tile_assets.wall_material.clone(),
                    TileType::Wall,
                ),
                _ => (
                    tile_assets.floor_mesh.clone(),
                    tile_assets.floor_material.clone(),
                    TileType::Floor,
                ),
            };

            commands
                .spawn((
                    Mesh2d(hover_mesh.clone()),
                    Transform::from_xyz(pos.x, pos.y, 1.0),
                    Tile::new(x as i32, y as i32, tile_type),
                    tile_overlays::TileOverlay::default(),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh2d(overlay_mesh.clone()),
                        MeshMaterial2d(overlay_materials.none.clone()),
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
                .observe(
                    |event: On<Pointer<Over>>,
                     units: Query<(Entity, &GridPosition), With<Unit>>,
                     tiles: Query<(Entity, &Tile)>,
                     action_range: Res<UnitActionRange>,
                     mut overlays: Query<&mut TileOverlay>| {
                        let entity = event.event_target();
                        if let Some((unit_entity, _)) = units.iter().find(|(_, pos)| {
                            **pos == GridPosition::from(tiles.get(entity).unwrap().1)
                        }) {
                            for (entity, tile) in tiles {
                                let valid_attack =
                                    action_range.attack_tiles[&unit_entity].contains(&tile.into());
                                let valid_move =
                                    action_range.move_tiles[&unit_entity].contains(&tile.into());

                                let mut overlay = overlays.get_mut(entity).unwrap();

                                if valid_attack {
                                    overlay.attack_transparent = true;
                                }
                                if valid_move {
                                    overlay.move_transparent = true;
                                }
                            }
                        }
                    },
                )
                .observe(update_overlay_material::<Pointer<Out>>(
                    OverlayLayer::Hover,
                    false,
                ))
                .observe(
                    |_: On<Pointer<Out>>,
                     tiles: Query<Entity, With<Tile>>,
                     mut overlays: Query<&mut TileOverlay>| {
                        for entity in tiles {
                            overlays.get_mut(entity).unwrap().attack_transparent = false;
                            overlays.get_mut(entity).unwrap().move_transparent = false;
                        }
                    },
                )
                .observe(
                    |event: On<Pointer<Click>>,
                     tiles: Query<(Entity, &Tile)>,
                     players: Query<
                        (Entity, &GridPosition),
                        (With<PlayerUnit>, Without<HasMoved>),
                    >,
                     mut ev_grid_clicked: MessageWriter<GridClicked>| {
                        let (entity, tile) = tiles.get(event.entity).unwrap();
                        let click_pos = tile.into();

                        if let Some((player, _)) =
                            players.iter().find(|(_, position)| **position == click_pos)
                        {
                            ev_grid_clicked.write(GridClicked {
                                tile: entity,
                                click_pos,
                                unit: Some(player),
                            });
                        } else {
                            ev_grid_clicked.write(GridClicked {
                                tile: entity,
                                click_pos,
                                unit: None,
                            });
                        };
                    },
                );
        }
    }
}

pub fn build_tilemap(mut commands: Commands, tiles: Query<(Entity, &Tile)>) {
    let map = tiles
        .iter()
        .map(|(entity, tile)| (GridPosition::from(tile), entity))
        .collect();
    commands.insert_resource(Tilemap(map));
}

pub fn los(from: GridPosition, to: GridPosition, tilemap: &Tilemap, tiles: &Query<&Tile>) -> bool {
    for pos in line(from, to) {
        if pos == from {
            continue;
        }
        if let Some(&entity) = tilemap.0.get(&pos)
            && let Ok(tile) = tiles.get(entity)
            && tile.r#type == TileType::Wall
        {
            return false;
        }
    }
    true
}

// https://rosettacode.org/wiki/Bitmap/Bresenham's_line_algorithm#Rust
fn line(from: GridPosition, to: GridPosition) -> Vec<GridPosition> {
    let mut coordinates: Vec<GridPosition> = vec![];

    let x1 = from.x;
    let y1 = from.y;
    let x2 = to.x;
    let y2 = to.y;

    let dx: i32 = i32::abs(x2 - x1);
    let dy: i32 = i32::abs(y2 - y1);
    let sx: i32 = if x1 < x2 { 1 } else { -1 };
    let sy: i32 = if y1 < y2 { 1 } else { -1 };

    let mut error: i32 = (if dx > dy { dx } else { -dy }) / 2;
    let mut current_x: i32 = x1;
    let mut current_y: i32 = y1;
    loop {
        coordinates.push(GridPosition {
            x: current_x,
            y: current_y,
        });

        if current_x == x2 && current_y == y2 {
            break;
        }

        let error2: i32 = error;

        if error2 > -dx {
            error -= dy;
            current_x += sx;
        }
        if error2 < dy {
            error += dx;
            current_y += sy;
        }
    }
    coordinates
}
