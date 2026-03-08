use std::ops::Sub;

use bevy::prelude::*;

use crate::{
    tile_overlays::{OverlayLayer, TileOverlay, TileOverlayMaterials, update_overlay_material},
    units::{
        Attack,
        enemy::EnemyUnit,
        player::{HasMoved, PlayerUnit},
    },
};

pub const TILE_SIZE: f32 = 64.0;
pub const MAP_WIDTH: u32 = 12;
pub const MAP_HEIGHT: u32 = 9;

const THICKNESS: f32 = 2.0;

#[derive(Message, Debug, Clone, Copy)]
pub struct GridClicked {
    pub tile: Entity,
    pub click_pos: GridPosition,
    pub unit: Option<Entity>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub overlay: TileOverlay,
}

impl Tile {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            overlay: TileOverlay::default(),
        }
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

#[derive(Component, Clone, Copy, PartialEq, Debug)]
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

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let overlay_materials = TileOverlayMaterials {
        none: materials.add(Color::NONE),
        r#move: materials.add(Color::srgba(0.0, 1.0, 0.0, 0.5)),
        attack: materials.add(Color::srgba(1.0, 0.0, 0.0, 0.5)),
        enemy_attack: materials.add(Color::srgba(1.0, 0.0, 0.0, 0.3)),
        move_attack: materials.add(Color::srgba(0.5, 0.5, 0.0, 0.5)),
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
                    Tile::new(x as i32, y as i32),
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
                     enemies: Query<(&GridPosition, &Attack), With<EnemyUnit>>,
                     tiles: Query<&mut Tile>| {
                        let entity = event.event_target();
                        if let Some((enemy_pos, attack)) = enemies.iter().find(|(pos, _)| {
                            **pos == GridPosition::from(tiles.get(entity).unwrap())
                        }) {
                            for mut tile in tiles {
                                if attack.range.contains(*enemy_pos, GridPosition::from(*tile)) {
                                    tile.overlay.enemy_attack = true;
                                }
                            }
                        }
                    },
                )
                .observe(update_overlay_material::<Pointer<Out>>(
                    OverlayLayer::Hover,
                    false,
                ))
                .observe(|_: On<Pointer<Out>>, tiles: Query<&mut Tile>| {
                    for mut tile in tiles {
                        tile.overlay.enemy_attack = false;
                    }
                })
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

    commands.insert_resource(overlay_materials);
}
