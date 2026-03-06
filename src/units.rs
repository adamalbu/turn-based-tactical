use bevy::prelude::*;

use crate::grid::{self, GridPosition};

#[derive(Component)]
pub struct PlayerUnit;

#[derive(Component)]
pub struct EnemyUnit;

#[derive(Component)]
pub struct Health {
    pub hp: u32,
    pub max_hp: u32,
}

#[derive(Component)]
pub struct Attack {
    pub damage: u32,
    pub range: u32,
}

pub enum MovementShape {
    Square(u32),
}

#[derive(Component)]
pub struct Movement {
    pub range: MovementShape,
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,

    query: Query<(&grid::Tile, &Transform)>,
) {
    const SPAWN_TILE: grid::Tile = grid::Tile { x: 0, y: 0 };

    dbg!(&query);

    let (_tile, tile_transform) = query
        .iter()
        .find(|(tile, _)| tile.x == SPAWN_TILE.x && tile.y == SPAWN_TILE.y)
        .expect("No tile exists there");

    let tile_pos = tile_transform.translation;

    let player_mesh = meshes.add(Circle::new(grid::TILE_SIZE / 2.5));
    let player_material = materials.add(Color::srgb(1.0, 0.0, 0.0));

    dbg!(&tile_pos);

    commands.spawn((
        Mesh2d(player_mesh),
        MeshMaterial2d(player_material),
        Transform::from_xyz(tile_pos.x, tile_pos.y, 0.0),
        PlayerUnit,
        GridPosition::from(SPAWN_TILE),
    ));
}
