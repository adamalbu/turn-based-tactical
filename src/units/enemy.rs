use bevy::prelude::*;

use std::time::Duration;

use crate::{
    GameState,
    grid::{GridPosition, Tile},
    units::{
        Attack, Health, HealthBarAssets, HealthBarForeground, Movement, RangeShape, Unit,
        player::PlayerUnit,
    },
};

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

pub fn on_enemy_turn(
    mut commands: Commands,
    enemies: Query<
        (
            Entity,
            &mut GridPosition,
            Option<&Movement>,
            Option<&mut Health>,
        ),
        With<EnemyUnit>,
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
