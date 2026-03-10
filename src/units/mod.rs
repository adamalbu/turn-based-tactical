use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

pub mod enemy;
pub mod player;

use crate::{
    game::{self},
    grid::{self, GridPosition, Tile, Tilemap, los},
    units::{enemy::EnemyAssets, player::PlayerAssets},
};

#[derive(Resource, Default)]
pub struct UnitActionRange {
    pub move_tiles: HashMap<Entity, HashSet<GridPosition>>,
    pub attack_tiles: HashMap<Entity, HashSet<GridPosition>>,
}

#[derive(Resource, Default, Deref, DerefMut, Clone, Copy)]
pub struct SelectedUnit(pub Option<Entity>);

#[derive(Component)]
pub struct Unit;

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub hp: u32,
    max_hp: u32,
}

impl Health {
    pub fn new(hp: u32) -> Self {
        Health { hp, max_hp: hp }
    }
}

const HEALTH_BAR_WIDTH: f32 = 30.0;
const HEALTH_BAR_HEIGHT: f32 = 5.0;

#[derive(Resource, Default)]
pub struct HealthBarAssets {
    pub background_mesh: Handle<Mesh>,
    pub background_material: Handle<ColorMaterial>,
    pub health_mesh: Handle<Mesh>,
    pub health_material: Handle<ColorMaterial>,
}

#[derive(Component)]
pub struct HealthBarForeground;

#[derive(Component)]
pub struct Attack {
    pub damage: u32,
    pub range: RangeShape,
}

#[derive(Clone, Copy)]
pub enum RangeShape {
    Square(u32),
    Axis,
}

impl RangeShape {
    fn contains(self, origin: GridPosition, tile: GridPosition) -> bool {
        let dx = (tile.x - origin.x).abs();
        let dy = (tile.y - origin.y).abs();

        if dx == 0 && dy == 0 {
            return false;
        }

        match self {
            Self::Square(range) => dx < range as i32 && dy < range as i32,
            Self::Axis => dx == 0 || dy == 0,
        }
    }

    pub fn contains_with_los<F>(&self, origin: GridPosition, tile: GridPosition, los: F) -> bool
    where
        F: Fn(GridPosition, GridPosition) -> bool,
    {
        if !self.contains(origin, tile) {
            false
        } else {
            los(origin, tile)
        }
    }
}

#[derive(Component)]
pub struct Movement {
    pub range: RangeShape,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SelectedUnit>()
        .init_resource::<HealthBarAssets>()
        .init_resource::<UnitActionRange>()
        .add_systems(Startup, setup.after(grid::spawn))
        .add_systems(PreUpdate, calculate_ranges)
        .add_systems(Update, (update_positions, update_health_bar));
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_assets: ResMut<PlayerAssets>,
    mut enemy_assets: ResMut<EnemyAssets>,
    mut health_bar_assets: ResMut<HealthBarAssets>,
) {
    player_assets.mesh = meshes.add(Circle::new(grid::TILE_SIZE / 2.5));
    player_assets.material = materials.add(Color::srgb(1.0, 0.0, 0.0));

    enemy_assets.mesh = meshes.add(Circle::new(grid::TILE_SIZE / 2.5));
    enemy_assets.material = materials.add(Color::srgb(0.0, 1.0, 0.0));

    let bar_mesh = meshes.add(Rectangle::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT));
    *health_bar_assets = HealthBarAssets {
        background_mesh: bar_mesh.clone(),
        background_material: materials.add(Color::srgb(0.2, 0.2, 0.2)),
        health_mesh: bar_mesh,
        health_material: materials.add(Color::srgb(1.0, 0.0, 0.0)),
    };

    let player_assets = &player_assets.into();
    let enemy_assets = &enemy_assets.into();
    let health_bar_assets = &health_bar_assets.into();

    for (y, row) in game::LEVEL.iter().enumerate() {
        for (x, char) in row.chars().enumerate() {
            match char {
                'P' => {
                    player::spawn(
                        GridPosition {
                            x: x as i32,
                            y: y as i32,
                        },
                        &mut commands,
                        player_assets,
                        health_bar_assets,
                    );
                }
                'E' => {
                    enemy::spawn(
                        GridPosition {
                            x: x as i32,
                            y: y as i32,
                        },
                        &mut commands,
                        enemy_assets,
                        health_bar_assets,
                    );
                }
                _ => continue,
            }
        }
    }
}

pub fn update_positions(
    transforms: Query<(&mut Transform, &GridPosition), (With<Unit>, Changed<GridPosition>)>,
    tiles: Query<(&grid::Tile, &Transform), Without<Unit>>,
) {
    for (mut transform, grid_pos) in transforms {
        let (_tile, tile_transform) = tiles
            .iter()
            .find(|(tile, _)| tile.x == grid_pos.x && tile.y == grid_pos.y)
            .expect("No tile exists there");

        let tile_pos = tile_transform.translation;

        transform.translation = Vec3::new(tile_pos.x, tile_pos.y, 0.0);
    }
}

pub fn update_health_bar(
    units: Query<(&Children, &Health), Changed<Health>>,
    mut transforms: Query<&mut Transform, With<HealthBarForeground>>,
) {
    for (children, health) in units {
        for child in children {
            if let Ok(mut transform) = transforms.get_mut(*child) {
                let ratio = health.hp as f32 / health.max_hp as f32;
                transform.scale.x = ratio;
                transform.translation.x = -(((1.0 - ratio) * HEALTH_BAR_WIDTH) / 2.0);
            }
        }
    }
}

pub fn calculate_ranges(
    mut action_range: ResMut<UnitActionRange>,
    units: Query<(Entity, &GridPosition, Option<&Movement>, Option<&Attack>), With<Unit>>,
    unit_positions: Query<
        &GridPosition,
        (With<Unit>, Or<(Changed<GridPosition>, Added<GridPosition>)>),
    >,
    tiles: Query<&Tile>,
    tilemap: ResMut<Tilemap>,
) {
    if unit_positions.is_empty() {
        return;
    }

    action_range.move_tiles.clear();
    action_range.attack_tiles.clear();

    for (entity, pos, movement, attack) in units {
        for tile in tiles {
            let tile_pos = tile.into();
            if let Some(movement) = movement
                && movement
                    .range
                    .contains_with_los(*pos, tile_pos, |from, to| los(from, to, &tilemap, &tiles))
            {
                let is_over_unit = unit_positions.iter().any(|unit_pos| unit_pos == &tile_pos);
                if !is_over_unit {
                    action_range
                        .move_tiles
                        .entry(entity)
                        .or_default()
                        .insert(tile_pos);
                }
            }

            if let Some(attack) = attack
                && attack
                    .range
                    .contains_with_los(*pos, tile_pos, |from, to| los(from, to, &tilemap, &tiles))
            {
                action_range
                    .attack_tiles
                    .entry(entity)
                    .or_default()
                    .insert(tile_pos);
            }
        }
    }
}
