use bevy::prelude::*;
use bevy::math::vec3;
use bevy::input::mouse::MouseWheel;
use bevy_pancam::PanCamPlugin;
use noise::{NoiseFn, Perlin};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Default, Resource)]
struct TileMap {
    tiles: HashMap<(i32, i32), TileType>,
    spawned: HashSet<(i32, i32)>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
enum TileType {
    Grass,
    Water,
    Mountain,
}

#[derive(Component, Serialize, Deserialize)]
struct SerializableTile {
    x: i32,
    y: i32,
    tile_type: TileType,
}

const TILE_SIZE: f32 = 32.0;
const VIEW_RADIUS: i32 = 20;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgba(1.0, 0.92, 0.9, 1.0)))
        .insert_resource(TileMap::default())
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (800., 600.).into(),
                title: "Tile Map!".into(),
                ..default()
            }),
            ..default()
        }), PanCamPlugin::default()))
        .add_systems(Startup, init_app)
        .add_systems(Update, (camera_movement, update_tiles))
        .run();
}

fn camera_movement(
    mut cam_query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut scroll_evr: EventReader<MouseWheel>,
    time: Res<Time>,
) {
    let (mut transform, mut projection) = cam_query.single_mut();

    let mut direction = Vec3::ZERO;
    let speed = 500.0;
    let delta = time.delta_seconds();

    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    transform.translation += direction.normalize_or_zero() * speed * delta;

    for ev in scroll_evr.read() {
        let zoom_change = ev.y * 0.1;
        projection.scale = (projection.scale - zoom_change).clamp(0.1, 10.0);
    }
}

fn init_app(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_tiles(
    mut commands: Commands,
    cam_query: Query<&Transform, With<Camera>>,
    mut tile_map: ResMut<TileMap>,
    tiles_query: Query<(Entity, &Transform), With<SerializableTile>>,
) {
    let cam_pos = cam_query.get_single().map_or(vec3(0.0, 0.0, -5.0), |t| t.translation);

    let center_x = (cam_pos.x / TILE_SIZE).round() as i32;
    let center_y = (cam_pos.y / TILE_SIZE).round() as i32;

    let visible_tiles: HashSet<(i32, i32)> = ((center_y - VIEW_RADIUS)..=(center_y + VIEW_RADIUS))
        .flat_map(|y| {
            (center_x - VIEW_RADIUS..=center_x + VIEW_RADIUS)
                .map(move |x| (x, y))
        })
        .collect();

    for (entity, transform) in tiles_query.iter() {
        let tile_x = (transform.translation.x / TILE_SIZE).round() as i32;
        let tile_y = (transform.translation.y / TILE_SIZE).round() as i32;
        let coord = (tile_x, tile_y);

        if !visible_tiles.contains(&coord) {
            commands.entity(entity).despawn();
            tile_map.spawned.remove(&coord);
        }
    }

    let perlin = Perlin::new(1);

    for &(x, y) in &visible_tiles {
        if tile_map.spawned.contains(&(x, y)) {
            continue;
        }

        let tile_type = tile_map.tiles.entry((x, y)).or_insert_with(|| {
            let noise = perlin.get([x as f64 / 10.0, y as f64 / 10.0]);
            match noise {
                n if n < -0.2 => TileType::Water,
                n if n < 0.4 => TileType::Grass,
                _ => TileType::Mountain,
            }
        });

        let color = match tile_type {
            TileType::Grass => Color::rgb(0.3, 1.0, 0.3),
            TileType::Water => Color::rgb(0.3, 0.3, 1.0),
            TileType::Mountain => Color::rgb(0.3, 0.3, 0.3),
        };

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(TILE_SIZE - 2.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE,
                    0.0,
                ),
                ..default()
            },
            SerializableTile {
                x,
                y,
                tile_type: *tile_type,
            },
        ));

        tile_map.spawned.insert((x, y));
    }
}
