use std::time::Duration;

use bevy::prelude::*;

use crate::{
    game,
    grid::GridPosition,
    interaction::{self, SelectedPosition},
    ui::PlayerAction,
    units::{
        Attack, Health, HealthBarAssets, HealthBarForeground, Movement, RangeShape, SelectedUnit,
        Unit, UnitActionRange,
        enemy::{self, EnemyUnit},
    },
};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnState {
    #[default]
    None,
    SelectedUnit,
    SelectedPosition,
    End,
}

#[derive(Component)]
pub struct PlayerUnit;

#[derive(Resource, Default)]
pub struct PlayerAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

#[derive(Component)]
pub struct HasActed;

pub fn plugin(app: &mut App) {
    app.init_resource::<PlayerAssets>()
        .init_state::<TurnState>()
        .init_resource::<interaction::SelectedPosition>()
        .add_observer(interaction::on_deselect)
        .add_systems(
            OnEnter(TurnState::SelectedUnit),
            interaction::selected_player.run_if(in_state(game::GameState::PlayerTurn)),
        )
        .add_systems(
            OnEnter(TurnState::None),
            (interaction::deselect, check_player_turn_over)
                .run_if(in_state(game::GameState::PlayerTurn)),
        )
        .add_systems(
            OnEnter(TurnState::SelectedPosition),
            interaction::selected_position,
        )
        .add_systems(OnEnter(TurnState::End), end_turn)
        .add_systems(OnEnter(game::GameState::PlayerTurn), on_player_turn)
        .add_systems(Update, (move_player, check_player_turn_over));
    bevy::log::debug!("player plugin");
}

pub fn spawn(
    spawn_pos: GridPosition,
    commands: &mut Commands,
    player_assets: &Res<PlayerAssets>,
    health_bar_assets: &Res<HealthBarAssets>,
) {
    commands
        .spawn((
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
            Health::new(8),
            spawn_pos,
        ))
        .with_children(|parent| {
            parent.spawn((
                HealthBarForeground,
                Mesh2d(health_bar_assets.health_mesh.clone()),
                MeshMaterial2d(health_bar_assets.health_material.clone()),
                Transform::from_xyz(0.0, -30.0, 0.91),
            ));
            parent.spawn((
                Mesh2d(health_bar_assets.background_mesh.clone()),
                MeshMaterial2d(health_bar_assets.background_material.clone()),
                Transform::from_xyz(0.0, -30.0, 0.9),
            ));
        });
}

pub fn check_player_turn_over(
    mut commands: Commands,
    mut next_turn: ResMut<NextState<TurnState>>,
    unactionable_units: Query<&PlayerUnit, Without<HasActed>>,
    actionable_units: Query<Entity, (With<HasActed>, With<PlayerUnit>)>,
) {
    if unactionable_units.count() == 0 {
        for entity in actionable_units {
            commands.entity(entity).remove::<HasActed>();
        }
        next_turn.set(TurnState::End);
    }
}

pub fn on_player_turn(
    mut commands: Commands,
    enemies: Query<(Entity, Option<&Attack>), With<EnemyUnit>>,
    players: Query<
        (Entity, &GridPosition, Option<&mut Health>),
        (With<PlayerUnit>, Without<EnemyUnit>),
    >,
    action_range: Res<UnitActionRange>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    bevy::platform::thread::sleep(Duration::from_millis(300));

    for (player_entity, player_pos, player_health) in players {
        if let Some(mut player_health) = player_health {
            for (enemy_entity, enemy_attack) in enemies {
                if let Some(enemy_attack) = enemy_attack
                    && action_range.attack_tiles[&enemy_entity].contains(player_pos)
                {
                    if player_health.hp <= enemy_attack.damage {
                        commands.entity(player_entity).despawn();
                    } else {
                        player_health.hp -= enemy_attack.damage;
                    }
                }
            }
        }
    }
}

pub fn move_player(
    mut commands: Commands,
    selected_unit: Res<SelectedUnit>,
    target_pos: Res<SelectedPosition>,
    mut player_transform: Query<&mut GridPosition, With<PlayerUnit>>,
    mut ev_player_action: MessageReader<PlayerAction>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    for ev in ev_player_action.read() {
        match ev {
            PlayerAction::Move => {
                let mut transform = player_transform.get_mut(selected_unit.unwrap()).unwrap();
                *transform = target_pos.0.unwrap();
            }
            PlayerAction::Wait => (),
        }

        commands.entity(selected_unit.unwrap()).insert(HasActed);

        next_state.set(TurnState::None);
    }
}

pub fn end_turn(
    mut next_player_turn: ResMut<NextState<TurnState>>,
    mut next_enemy_turn: ResMut<NextState<enemy::TurnState>>,
    mut next_state: ResMut<NextState<game::GameState>>,
) {
    next_player_turn.set(TurnState::None);
    next_enemy_turn.set(enemy::TurnState::None);
    next_state.set(game::GameState::EnemyTurn);
}
