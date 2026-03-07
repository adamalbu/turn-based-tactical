use bevy::prelude::*;

use crate::{
    PlayerTurnState,
    grid::{self, GridPosition},
    interaction::SelectedPosition,
    ui::MoveButtonClicked,
};

#[derive(Resource, Default, Deref, DerefMut, Clone, Copy)]
pub struct SelectedUnit(pub Option<Entity>);

#[derive(Component, Debug, Clone, Copy)]
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

#[derive(Clone, Copy)]
pub enum MoveShape {
    Square(u32),
}

impl MoveShape {
    pub fn contains(self, origin: GridPosition, tile: GridPosition) -> bool {
        let dx = (tile.x - origin.x).abs();
        let dy = (tile.y - origin.y).abs();

        if dx == 0 && dy == 0 {
            return false;
        }

        match self {
            Self::Square(range) => dx < range as i32 && dy < range as i32,
        }
    }
}

#[derive(Component)]
pub struct Movement {
    pub range: MoveShape,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_assets: ResMut<PlayerAssets>,
) {
    player_assets.mesh = meshes.add(Circle::new(grid::TILE_SIZE / 2.5));
    player_assets.material = materials.add(Color::srgb(1.0, 0.0, 0.0));

    let player_assets = &player_assets.into();

    spawn_player(GridPosition { x: 0, y: 0 }, &mut commands, player_assets);
    spawn_player(GridPosition { x: 4, y: 3 }, &mut commands, player_assets);
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
        PlayerUnit,
        Movement {
            range: MoveShape::Square(3),
        },
        spawn_pos,
    ));
}

pub fn update_positions(
    transforms: Query<(&mut Transform, &GridPosition), (With<PlayerUnit>, Changed<GridPosition>)>,
    tiles: Query<(&grid::Tile, &Transform), Without<PlayerUnit>>,
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
    actionable_units: Query<&PlayerUnit, Without<HasMoved>>,
) {
    for _ in ev_move_clicked.read() {
        dbg!(actionable_units.count());
    }
}
