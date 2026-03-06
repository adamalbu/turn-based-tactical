use bevy::prelude::*;

const TILE_SIZE: f32 = 64.0;
const THICKNESS: f32 = 2.0;
const WIDTH: u32 = 12;
const HEIGHT: u32 = 9;

#[derive(Component)]
struct Tile {
    x: i32,
    y: i32,
}

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let tile_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE).to_ring(THICKNESS));
    let tile_material = materials.add(Color::BLACK);

    let offset = Vec2::new(
        -(WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
        -(HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
    );

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let pos = Vec2::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE) + offset;
            dbg!(&pos);

            commands.spawn((
                Mesh2d(tile_mesh.clone()),
                MeshMaterial2d(tile_material.clone()),
                Transform::from_xyz(pos.x, pos.y, 0.0),
                Tile {
                    x: x as i32,
                    y: y as i32,
                },
            ));
        }
    }
}
