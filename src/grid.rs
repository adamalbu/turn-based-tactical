use bevy::prelude::*;

use crate::units::{self, MoveShape, PlayerSelected, PlayerUnit};

pub const TILE_SIZE: f32 = 64.0;
pub const MAP_WIDTH: u32 = 12;
pub const MAP_HEIGHT: u32 = 9;

const THICKNESS: f32 = 2.0;

#[derive(Component)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
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
#[derive(Component)]
pub struct TileHighlight;

#[derive(Resource)]
pub struct TileOverlayMaterials {
    none: Handle<ColorMaterial>,
    hover: Handle<ColorMaterial>,
    range: Handle<ColorMaterial>,
}

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let overlay_materials = TileOverlayMaterials {
        none: materials.add(Color::NONE),
        hover: materials.add(Color::srgba(1.0, 1.0, 0.0, 0.5)),
        range: materials.add(Color::srgba(0.0, 1.0, 0.0, 0.5)),
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
                    Tile {
                        x: x as i32,
                        y: y as i32,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh2d(overlay_mesh.clone()),
                        MeshMaterial2d(overlay_materials.none.clone()),
                        TileHighlight,
                        Transform::from_xyz(0.0, 0.0, -0.1),
                    ));

                    parent.spawn((
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(tile_material.clone()),
                        Transform::from_xyz(0.0, 0.0, -0.2),
                    ));
                })
                .observe(update_overlay_material::<Pointer<Over>>(
                    overlay_materials.hover.clone(),
                ))
                .observe(update_overlay_material::<Pointer<Out>>(
                    overlay_materials.none.clone(),
                ))
                .observe(
                    |event: On<Pointer<Click>>,
                     tiles: Query<&Tile>,
                     players: Query<(Entity, &GridPosition), With<PlayerUnit>>,
                     mut ev_player_selected: MessageWriter<units::PlayerSelected>| {
                        let clicked_coords: GridPosition = tiles.get(event.entity).unwrap().into();

                        if let Some((player, _)) = players
                            .iter()
                            .find(|(_, position)| **position == clicked_coords)
                        {
                            ev_player_selected.write(units::PlayerSelected(player));
                            println!("clicked player");
                        };
                    },
                );
        }
    }

    commands.insert_resource(overlay_materials);
}

#[allow(clippy::type_complexity)]
fn update_overlay_material<E: EntityEvent>(
    new_material: Handle<ColorMaterial>,
) -> impl Fn(
    On<E>,
    Query<&Children, With<Tile>>,
    Query<&mut MeshMaterial2d<ColorMaterial>, With<TileHighlight>>,
) {
    move |event, query, mut highlights| {
        let children = query.get(event.event_target()).unwrap();
        let child = children.first().unwrap();

        let mut material = highlights.get_mut(*child).unwrap();
        *material = MeshMaterial2d(new_material.clone());
    }
}

pub fn show_player_move_range(
    mut ev_player_selected: MessageReader<PlayerSelected>,
    query: Query<(&GridPosition, &units::Movement), With<PlayerUnit>>,
    tiles: Query<(&Children, &Tile)>,
    mut highlights: Query<&mut MeshMaterial2d<ColorMaterial>, With<TileHighlight>>,
    overlay_material: Res<TileOverlayMaterials>,
) {
    for ev in ev_player_selected.read() {
        dbg!(&ev);
        let (origin, movement) = query.get(ev.0).unwrap();
        show_move_range(
            *origin,
            movement.range,
            tiles,
            &mut highlights,
            &overlay_material,
        );
    }
}

pub fn show_move_range(
    origin: GridPosition,
    range: MoveShape,
    tiles: Query<(&Children, &Tile)>,
    highlights: &mut Query<&mut MeshMaterial2d<ColorMaterial>, With<TileHighlight>>,
    overlay_material: &Res<TileOverlayMaterials>,
) {
    let range_material = overlay_material.range.clone();

    for (children, tile) in tiles {
        if range.contains(origin, tile.into()) {
            let child = children.first().unwrap();
            let mut material = highlights.get_mut(*child).unwrap();
            *material = MeshMaterial2d(range_material.clone())
        }
    }
}
