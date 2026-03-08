use std::{i32, time::Duration};

use bevy::prelude::*;

use crate::{
    GameState, PlayerTurnState,
    grid::{self, GridPosition, Tile},
    interaction::SelectedPosition,
    ui::MoveButtonClicked,
};

#[derive(Resource, Default, Deref, DerefMut, Clone, Copy)]
pub struct SelectedUnit(pub Option<Entity>);

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct PlayerUnit;

#[derive(Resource, Default)]
pub struct PlayerAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

#[derive(Component)]
pub struct HasMoved;

#[derive(Component)]
pub struct EnemyUnit;

#[derive(Resource, Default)]
pub struct EnemyAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

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
    pub fn contains(self, origin: GridPosition, tile: GridPosition) -> bool {
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
}

#[derive(Component)]
pub struct Movement {
    pub range: RangeShape,
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

    spawn_player(GridPosition { x: 1, y: 2 }, &mut commands, player_assets);
    spawn_player(GridPosition { x: 1, y: 4 }, &mut commands, player_assets);
    spawn_player(GridPosition { x: 1, y: 6 }, &mut commands, player_assets);

    spawn_enemy(
        GridPosition { x: 10, y: 3 },
        &mut commands,
        enemy_assets,
        health_bar_assets,
    );
    spawn_enemy(
        GridPosition { x: 10, y: 5 },
        &mut commands,
        enemy_assets,
        health_bar_assets,
    );
}

pub fn spawn_player(
    spawn_pos: GridPosition,
    commands: &mut Commands,
    player_assets: &Res<PlayerAssets>,
) {
    commands.spawn((
        Mesh2d(player_assets.mesh.clone()),
        MeshMaterial2d(player_assets.material.clone()),
        Transform::default(),
        Unit,
        PlayerUnit,
        Movement {
            range: RangeShape::Square(3),
        },
        Attack {
            damage: 4,
            range: RangeShape::Axis,
        },
        spawn_pos,
    ));
}

pub fn spawn_enemy(
    spawn_pos: GridPosition,
    commands: &mut Commands,
    enemy_assets: &Res<EnemyAssets>,
    health_bar_assets: &Res<HealthBarAssets>,
) {
    commands
        .spawn((
            Mesh2d(enemy_assets.mesh.clone()),
            MeshMaterial2d(enemy_assets.material.clone()),
            Transform::default(),
            Unit,
            EnemyUnit,
            Movement {
                range: RangeShape::Square(3),
            },
            Attack {
                damage: 3,
                range: RangeShape::Square(4),
            },
            Health::new(10),
            spawn_pos,
        ))
        .with_children(|parent| {
            parent.spawn((
                HealthBarForeground,
                Mesh2d(health_bar_assets.health_mesh.clone()),
                MeshMaterial2d(health_bar_assets.health_material.clone()),
                Transform::from_xyz(0.0, -30.0, 0.9),
            ));
            parent.spawn((
                Mesh2d(health_bar_assets.background_mesh.clone()),
                MeshMaterial2d(health_bar_assets.background_material.clone()),
                Transform::from_xyz(0.0, -30.0, 0.9),
            ));
        });
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

pub fn move_unit(
    mut commands: Commands,
    selected_unit: Res<SelectedUnit>,
    target_pos: Res<SelectedPosition>,
    mut player_transform: Query<&mut GridPosition, With<PlayerUnit>>,
    mut ev_move_clicked: MessageReader<MoveButtonClicked>,
    mut next_state: ResMut<NextState<PlayerTurnState>>,
) {
    for _ in ev_move_clicked.read() {
        let mut transform = player_transform.get_mut(selected_unit.unwrap()).unwrap();
        *transform = target_pos.0.unwrap();

        commands.entity(selected_unit.unwrap()).insert(HasMoved);

        next_state.set(PlayerTurnState::None);
    }
}

pub fn handle_player_turn(
    mut ev_move_clicked: MessageReader<MoveButtonClicked>,
    mut next_state: ResMut<NextState<GameState>>,
    actionable_units: Query<&PlayerUnit, Without<HasMoved>>,
) {
    for _ in ev_move_clicked.read() {
        if actionable_units.count() == 0 {
            next_state.set(GameState::EnemyTurn)
        }
    }
}

pub fn check_win(enemies: Query<&EnemyUnit>, mut next_state: ResMut<NextState<GameState>>) {
    if enemies.count() == 0 {
        next_state.set(GameState::Win);
    }
}

pub fn on_enemy_turn(
    mut commands: Commands,
    enemies: Query<
        (
            Entity,
            &mut GridPosition,
            Option<&Movement>,
            Option<&mut Health>,
        ),
        (With<EnemyUnit>, Without<HasMoved>),
    >,
    players: Query<(&GridPosition, &Attack), (With<PlayerUnit>, Without<EnemyUnit>)>,
    tiles: Query<(Entity, &Tile)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    bevy::platform::thread::sleep(Duration::from_millis(500));

    for (entity, mut pos, movement, mut health) in enemies {
        for (player_pos, attack) in players {
            if attack.range.contains(*player_pos, *pos)
                && let Some(ref mut health) = health
            {
                if health.hp <= attack.damage {
                    commands.entity(entity).despawn();
                } else {
                    health.hp -= attack.damage;
                }
            }
        }

        let (target, _) = players
            .iter()
            .min_by_key(|(pp, _)| (pp.x - pos.x).abs() + (pp.y - pos.y).abs())
            .unwrap();

        if let Some(movement) = movement {
            let move_to = tiles
                .iter()
                .filter(|(_, tile)| movement.range.contains(*pos, GridPosition::from(*tile)))
                .min_by_key(|(_, tile)| (tile.x - target.x).abs() + (tile.y - target.y).abs())
                .unwrap();

            *pos = GridPosition::from(*(move_to.1));
        }
    }

    next_state.set(GameState::PlayerTurn);
}
