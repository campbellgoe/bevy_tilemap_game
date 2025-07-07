use bevy::prelude::*;
use bevy::math::vec3;
// We will use ButtonInput from prelude, so this comment is fine
// use bevy::input::mouse::MouseWheel;
use bevy_pancam::{DirectionKeys, PanCam, PanCamPlugin};
use noise::{NoiseFn, Simplex, Perlin};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// Needed for the camera setup

#[derive(Component)]
struct OrthoCamera;

#[derive(Component)]
struct PerspCamera;

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
const VIEW_RADIUS: i32 = 60;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.9, 0.92, 1.0)))
        .insert_resource(TileMap::default())
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
        .add_systems(Startup, init_app) // Use our modified init_app
        // Add both systems to Update
        .add_systems(Update, (update_tiles, toggle_camera_projection))
        .run();
}

// Modified init_app to set up both cameras
fn init_app(mut commands: Commands) {
    // Spawn the 2D Orthographic Camera with PanCam
    // This one is active by default
    commands.spawn((
        Camera2d, // Use the bundle
        PanCam {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle],
            move_keys: DirectionKeys {
                up:     vec![KeyCode::KeyW],
                down:   vec![KeyCode::KeyS],
                left:   vec![KeyCode::KeyA],
                right:  vec![KeyCode::KeyD],
            },
            speed: 300.,
            enabled: true, // PanCam is enabled for the Ortho camera
            zoom_to_cursor: true,
            min_scale: 1.,
            max_scale: 4.,
            min_x: f32::NEG_INFINITY,
            max_x: f32::INFINITY,
            min_y: f32::NEG_INFINITY,
            max_y: f32::INFINITY,
        },
        OrthoCamera, // Add the marker
    ));

    // Spawn the 3D Perspective Camera
    // Position it looking down and slightly forward
    let camera_pos_3d = vec3(0.0, -50.0, 100.0); // Example starting position
    let camera_look_at_3d = vec3(0.0, 0.0, 0.0); // Example point to look at
    commands.spawn((
        Camera3d {
            transform: Transform::from_translation(camera_pos_3d)
                .looking_at(camera_look_at_3d, Vec3::Y),
            ..default()
        },
        PerspCamera, // Add the marker
    ));
}

// System to toggle camera activity
fn toggle_camera_projection(
  mut query_ortho: Query<(&mut Visibility, Option<&mut PanCam>), With<OrthoCamera>>,
  mut query_persp: Query<&mut Visibility, With<PerspCamera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) { // Choose your toggle key
        let (mut ortho_visibility, mut pancam_option) = query_ortho.single_mut();
        let mut persp_visibility = query_persp.single_mut();
ortho_visibility.is_visible = !ortho_visibility.is_visible;
persp_visibility.is_visible = !persp_visibility.is_visible;


        if let Some(mut pancam) = pancam_option {
            pancam.enabled = ortho_visibility.is_visible
        ;
        }

        // Optional: You might want to reset the perspective camera's position
        // or sync it somehow with the 2D view position when toggling.
        // For simplicity, this example just swaps.
    }
}


fn update_tiles(
    mut commands: Commands,
    // Query specifically for the OrthoCamera's transform (assuming it drives tile spawning)
    // We assume the toggle system ensures the OrthoCamera is active when we want to update tiles based on its view.
    cam_query: Query<&Transform, With<OrthoCamera>>,
    mut tile_map: ResMut<TileMap>,
    tiles_query: Query<(Entity, &Transform), With<SerializableTile>>,
) {
    // Use .single() directly as we expect only one OrthoCamera
    let cam_transform = cam_query.single();
    let cam_pos = cam_transform.unwrap().translation;

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

    let simplex = Simplex::new(1000);
    let perlin = Perlin::new(1000);
    for &(x, y) in &visible_tiles {
        if tile_map.spawned.contains(&(x, y)) {
            continue;
        }

        let noise = (simplex.get([x as f64 / 10.0, y as f64 / 10.0]) + perlin.get([x as f64 / 10.0, y as f64 / 10.0]))/2.;
        let tile_type = tile_map.tiles.entry((x, y)).or_insert_with(|| {
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
                // FIX: Make size slightly larger to prevent lines
                custom_size: Some(Vec2::splat(TILE_SIZE + 0.1)),
                ..default()
            },
            Transform::from_xyz(
                x as f32 * TILE_SIZE,
                y as f32 * TILE_SIZE,
                0.0, // Z is 0 for 2D sprites
            ),
            SerializableTile {
                x,
                y,
                tile_type: *tile_type,
            }
        ));

        tile_map.spawned.insert((x, y));
    }
}