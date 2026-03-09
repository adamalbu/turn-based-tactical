use bevy::prelude::*;

use std::collections::HashSet;
use std::time::Duration;

use crate::{
    game,
    grid::{GridPosition, Tile},
    units::{
        Attack, Health, HealthBarAssets, HealthBarForeground, Movement, RangeShape, Unit,
        UnitActionRange,
        player::{self, PlayerUnit},
    },
};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnState {
    #[default]
    None,
    TakeDamage,
    Move,
    End,
}

#[derive(Component)]
pub struct EnemyUnit;

#[derive(Resource, Default)]
pub struct EnemyAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

pub fn spawn(
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

pub fn on_enemy_turn(mut next_turn: ResMut<NextState<TurnState>>) {
    bevy::platform::thread::sleep(Duration::from_millis(500));

    next_turn.set(TurnState::TakeDamage);
}

pub fn take_damage(
    mut commands: Commands,
    enemies: Query<(Entity, &GridPosition, Option<&mut Health>), With<EnemyUnit>>,
    players: Query<(Entity, &Attack), (With<PlayerUnit>, Without<EnemyUnit>)>,
    action_range: Res<UnitActionRange>,
    mut next_turn: ResMut<NextState<TurnState>>,
) {
    for (entity, pos, mut health) in enemies {
        for (player_entity, attack) in players {
            if action_range.attack_tiles[&player_entity].contains(pos)
                && let Some(ref mut health) = health
            {
                if health.hp <= attack.damage {
                    commands.entity(entity).despawn();
                } else {
                    health.hp -= attack.damage;
                }
            }
        }
    }

    next_turn.set(TurnState::Move);
}

pub fn r#move(
    mut enemies: Query<(Entity, &mut GridPosition), With<EnemyUnit>>,
    players: Query<&GridPosition, (With<PlayerUnit>, Without<EnemyUnit>)>,
    tiles: Query<(Entity, &Tile)>,
    action_range: Res<UnitActionRange>,
    mut next_turn: ResMut<NextState<TurnState>>,
) {
    let mut occupied: HashSet<(i32, i32)> = enemies.iter().map(|(_, pos)| (pos.x, pos.y)).collect();

    for (entity, mut pos) in &mut enemies {
        let target = players
            .iter()
            .min_by_key(|pp| (pp.x - pos.x).abs() + (pp.y - pos.y).abs())
            .unwrap();

        let move_to = tiles
            .iter()
            .filter(|(_, tile)| {
                let tp = GridPosition::from(*tile);
                action_range.move_tiles[&entity].contains(&tp)
                    && !players.iter().any(|pp| *pp == tp)
                    && !occupied.contains(&(tp.x, tp.y))
            })
            .min_by_key(|(_, tile)| (tile.x - target.x).abs() + (tile.y - target.y).abs());

        if let Some((_, tile)) = move_to {
            // remove old position, insert new one
            occupied.remove(&(pos.x, pos.y));
            *pos = GridPosition::from(*tile);
            occupied.insert((pos.x, pos.y));
        }
    }

    next_turn.set(TurnState::End);
}

pub fn end_turn(
    mut next_player_turn: ResMut<NextState<player::TurnState>>,
    mut next_enemy_turn: ResMut<NextState<TurnState>>,
    mut next_state: ResMut<NextState<game::GameState>>,
) {
    next_player_turn.set(player::TurnState::None);
    next_enemy_turn.set(TurnState::None);
    next_state.set(game::GameState::PlayerTurn);
}
