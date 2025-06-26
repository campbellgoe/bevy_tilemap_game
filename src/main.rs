use bevy::prelude::*;
use bevy::math::vec3;
use bevy_pancam::{DirectionKeys, PanCam, PanCamPlugin};
use noise::{NoiseFn, Simplex, Perlin};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Default, Resource)]
struct TileMap {
    tiles: HashMap<(i32, i32), TileType>,
    spawned: HashSet<(i32, i32)>,
}

#[derive(Resource, PartialEq, Debug)]
enum EditorMode {
    Pan,
    Paint,
}

#[derive(Resource)]
struct SelectedTileType(TileType);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
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
const VIEW_RADIUS: i32 = 60;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(1.0, 0.92, 0.9)))
        .insert_resource(TileMap::default())
        .insert_resource(EditorMode::Pan)
        .insert_resource(SelectedTileType(TileType::Grass))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (800., 600.).into(),
                    title: "Tile Map!".into(),
                    ..default()
                }),
                ..default()
            }),
            PanCamPlugin::default(),
        ))
        .add_systems(Startup, init_app)
        .add_systems(Update, (
            update_tiles,
            toggle_mode,
            paint_tiles,
            update_camera_control,
            switch_tile_type,
        ))
        .run();
}

fn init_app(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        PanCam {
            grab_buttons: vec![MouseButton::Left],
            move_keys: DirectionKeys {      // the keyboard buttons used to move the camera
                up:    vec![KeyCode::KeyW], // initalize the struct like this or use the provided methods for
                down:  vec![KeyCode::KeyS], // common key combinations
                left:  vec![KeyCode::KeyA],
                right: vec![KeyCode::KeyD],
            },
            enabled: true,
            ..default()
        },
    ));
}

fn toggle_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<EditorMode>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        *mode = match *mode {
            EditorMode::Pan => EditorMode::Paint,
            EditorMode::Paint => EditorMode::Pan,
        };
        info!("Switched to {:?} mode", *mode);
    }
}

fn update_camera_control(
    mode: Res<EditorMode>,
    mut query: Query<&mut PanCam>,
) {
    if mode.is_changed() {
        let pan_cam = query.single_mut();
        pan_cam.unwrap().enabled = *mode == EditorMode::Pan;
    }
}

fn switch_tile_type(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedTileType>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        selected.0 = TileType::Grass;
        info!("Switched to Grass");
    } else if keys.just_pressed(KeyCode::Digit2) {
        selected.0 = TileType::Water;
        info!("Switched to Water");
    } else if keys.just_pressed(KeyCode::Digit3) {
        selected.0 = TileType::Mountain;
        info!("Switched to Mountain");
    }
}

fn paint_tiles(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<EditorMode>,
    mut tile_map: ResMut<TileMap>,
    selected_tile: Res<SelectedTileType>,
    mut commands: Commands,
    tiles_query: Query<(Entity, &SerializableTile)>,
) {
    if *mode != EditorMode::Paint {
        return;
    }

    // Handle potential query errors
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_q.single() else { return };

    if let Some(screen_pos) = window.cursor_position() {
        if buttons.pressed(MouseButton::Left) {
            if let Ok(world_pos) = camera.viewport_to_world(camera_transform, screen_pos) {
                let world_pos = world_pos;

                let tile_x = (world_pos.origin.x / TILE_SIZE).round() as i32;
                let tile_y = (world_pos.origin.y / TILE_SIZE).round() as i32;
                let coord = (tile_x, tile_y);

                // Paint the tile
                tile_map.tiles.insert(coord, selected_tile.0);

                // Despawn old tile if it exists
                if let Some((entity, _)) = tiles_query.iter().find(|(_, tile)| (tile.x, tile.y) == coord) {
                    commands.entity(entity).despawn();
                    tile_map.spawned.remove(&coord);
                }
            }
        }
    }
}

fn update_tiles(
    mut commands: Commands,
    cam_query: Query<&Transform, With<Camera>>,
    mut tile_map: ResMut<TileMap>,
    tiles_query: Query<(Entity, &Transform), With<SerializableTile>>,
) {
    let cam_pos = cam_query.single().map_or(vec3(0.0, 0.0, -5.0), |t| t.translation);
    let center_x = (cam_pos.x / TILE_SIZE).round() as i32;
    let center_y = (cam_pos.y / TILE_SIZE).round() as i32;

    let visible_tiles: HashSet<(i32, i32)> = ((center_y - VIEW_RADIUS)..=(center_y + VIEW_RADIUS))
        .flat_map(|y| (center_x - VIEW_RADIUS..=center_x + VIEW_RADIUS).map(move |x| (x, y)))
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

    let simplex = Simplex::new(1000);
    let perlin = Perlin::new(1000);

    for &(x, y) in &visible_tiles {
        if tile_map.spawned.contains(&(x, y)) {
            continue;
        }

        let tile_type = tile_map.tiles.entry((x, y)).or_insert_with(|| {
            let noise = (simplex.get([x as f64 / 10.0, y as f64 / 10.0]) + perlin.get([x as f64 / 10.0, y as f64 / 10.0])) / 2.0;
            match noise {
                n if n < -0.2 => TileType::Water,
                n if n < 0.4 => TileType::Grass,
                _ => TileType::Mountain,
            }
        });

        let color = match tile_type {
            TileType::Grass => Color::srgb(0.3, 1.0, 0.3),
            TileType::Water => Color::srgb(0.0, 0.3, 1.0),
            TileType::Mountain => Color::srgb(0.3, 0.3, 0.3),
        };

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(TILE_SIZE - 0.1)),
                ..default()
            },
            Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0),
            SerializableTile {
                x,
                y,
                tile_type: *tile_type,
            },
        ));

        tile_map.spawned.insert((x, y));
    }
}
