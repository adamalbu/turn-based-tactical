use bevy::{
    picking::hover::{self, Hovered},
    prelude::*,
};

pub const TILE_SIZE: f32 = 64.0;
pub const MAP_WIDTH: u32 = 12;
pub const MAP_HEIGHT: u32 = 9;

const THICKNESS: f32 = 2.0;

#[derive(Component)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct TileHighlight;

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let hover_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE));

    let tile_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE).to_ring(THICKNESS));
    let tile_material = materials.add(Color::BLACK);

    let highlight_mesh = meshes.add(Rectangle::new(
        TILE_SIZE - THICKNESS * 2.0,
        TILE_SIZE - THICKNESS * 2.0,
    ));
    let highlight_material = materials.add(Color::srgba(1.0, 0.93, 0.34, 0.5));

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
                        Mesh2d(highlight_mesh.clone()),
                        MeshMaterial2d(highlight_material.clone()),
                        TileHighlight,
                        Transform::from_xyz(0.0, 0.0, -0.1),
                        Visibility::Hidden,
                    ));

                    parent.spawn((
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(tile_material.clone()),
                        Transform::from_xyz(0.0, 0.0, -0.2),
                    ));
                })
                .observe(
                    |mut over: On<Pointer<Over>>,
                     query: Query<&Children, With<Tile>>,
                     mut highlights: Query<&mut Visibility, With<TileHighlight>>| {
                        let children = query.get(over.entity).unwrap();
                        let child = children.first().unwrap();

                        let mut vis = highlights.get_mut(*child).unwrap();
                        *vis = Visibility::Visible;

                        over.propagate(false);
                    },
                ).observe(
                    |mut out: On<Pointer<Out>>,
                     query: Query<&Children, With<Tile>>,
                     mut highlights: Query<&mut Visibility, With<TileHighlight>>| {
                        let children = query.get(out.entity).unwrap();
                        let child = children.first().unwrap();

                        let mut vis = highlights.get_mut(*child).unwrap();
                        *vis = Visibility::Hidden;

                        out.propagate(false);
                    },
                );
        }
    }
}
