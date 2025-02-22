use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::render::camera::ScalingMode;
use bevy::window::MonitorSelection;
use bevy::sprite::TextureAtlas;

mod components;
mod map;
mod rendering;
mod input;
mod ui;
mod visibility;
mod systems;

// Camera control component
#[derive(Component)]
struct CameraControl {
    current_zoom: f32,
    target_zoom: f32,
    zoom_speed: f32,
}

impl Default for CameraControl {
    fn default() -> Self {
        Self {
            current_zoom: 1.0,
            target_zoom: 0.6,
            zoom_speed: 2.0,
        }
    }
}

use crate::components::{Position, Player, Tile, Npc, DialogBox}; 
use rand::seq::SliceRandom;
use crate::map::{TileMap, MAP_WIDTH, MAP_HEIGHT, TileType};
use crate::systems::check_dialog_distance;
use bevy::sprite::TextureAtlasSprite;
use crate::input::InputState;
use crate::visibility::{PlayerVisibility, update_visibility, update_tile_visibility, setup_visibility_map};
use bevy::text::{Text2dBundle, Text, TextStyle, TextAlignment};

const TILE_SIZE: f32 = 32.0;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    InGame,
}

#[derive(Resource)]
struct GameAssets {
    character_sheet: Handle<TextureAtlas>,
    floor_tiles: Handle<TextureAtlas>,
}


fn main() {
    App::new()
        .add_event::<RegenerateMapEvent>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Chasm".into(),
                resolution: (
                    MAP_WIDTH as f32 * TILE_SIZE,
                    MAP_HEIGHT as f32 * TILE_SIZE,
                ).into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                resizable: false,
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .add_state::<GameState>()
        .init_resource::<InputState>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::InGame), (
            spawn_game_world,
            setup_visibility_map.after(spawn_game_world)
        ))
        .add_systems(
            Update,
            (
                crate::input::handle_input,
                update_camera_zoom,
                update_sprite_positions.after(crate::input::handle_input),
                update_visibility.after(crate::input::move_player),
                crate::input::move_player.after(crate::input::handle_input),
                check_dialog_distance.after(crate::input::move_player),
                update_tile_visibility.after(update_visibility),
                handle_npc_interaction,
                render_dialog_boxes,
                regenerate_map_system.after(crate::input::handle_input),
                spawn_game_world
                    .after(regenerate_map_system)
                    .run_if(resource_exists::<TileMap>())
                    .run_if(on_event::<RegenerateMapEvent>()),
            ).run_if(in_state(GameState::InGame))
        )
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Camera
    let map = TileMap::new();
    let spawn_pos = map.get_spawn_position();
    let mut camera = Camera2dBundle::default();
    camera.transform.translation = Vec3::new(
        spawn_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
        spawn_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
        999.9
    );
    camera.projection.scaling_mode = ScalingMode::Fixed {
        width: MAP_WIDTH as f32 * TILE_SIZE,
        height: MAP_HEIGHT as f32 * TILE_SIZE,
    };
    camera.projection.scale = 1.0;
    commands.spawn((
        camera,
        CameraControl::default(),
    ));
    
    // Load character sprite sheet
    let character_handle = asset_server.load("sprites/rogues.png");
    let character_atlas = TextureAtlas::from_grid(
        character_handle,
        Vec2::new(32.0, 32.0),  // assuming 32x32 sprites
        16, 16,                 // 16x16 grid
        None, None
    );
    let character_atlas_handle = texture_atlases.add(character_atlas);

    // Load tiles sprite sheet
    let tiles_handle = asset_server.load("sprites/tiles.png");
    let tiles_atlas = TextureAtlas::from_grid(
        tiles_handle,
        Vec2::new(32.0, 32.0),  // assuming 32x32 sprites
        16, 16,                 // 16x16 grid
        None, None
    );
    let tiles_atlas_handle = texture_atlases.add(tiles_atlas);

    // Store the atlas handles as a resource
    commands.insert_resource(GameAssets {
        character_sheet: character_atlas_handle,
        floor_tiles: tiles_atlas_handle,
    });

    // Create initial TileMap
    commands.insert_resource(map);
}

fn spawn_game_world(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    map: Res<TileMap>,
    existing_entities: Query<Entity, Or<(With<Tile>, With<Player>)>>,
) {
    // First, clean up any existing entities
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn();
    }

    // Then spawn new tiles and player
    rendering::spawn_tiles(&mut commands, &map, &game_assets);

    // Find valid floor tiles for NPC spawn
    let floor_tiles: Vec<(i32, i32)> = (0..MAP_WIDTH as usize * MAP_HEIGHT as usize)
        .filter(|&i| {
            let row = i / MAP_WIDTH as usize; 
            let col = i % MAP_WIDTH as usize;
            map.tiles[row][col] == TileType::Floor
        })
        .map(|i| (
            (i % MAP_WIDTH as usize) as i32,
            (i / MAP_WIDTH as usize) as i32
        ))
        .collect();
        
    println!("Found {} floor tiles for NPC spawning", floor_tiles.len());

    // Choose random position away from player spawn
    let spawn_pos = map.get_spawn_position();
    let npc_pos = floor_tiles.into_iter()
        .filter(|pos| {
            let dx = (pos.0 - spawn_pos.0 as i32).abs();
            let dy = (pos.1 - spawn_pos.1 as i32).abs();
            dx + dy > 5 // Minimum Manhattan distance from player
        })
        .collect::<Vec<_>>();
        
    println!("Found {} valid positions for NPC (minimum 5 tiles from player)", npc_pos.len());

    let npc_pos = npc_pos
        .choose(&mut rand::thread_rng())
        .copied()
        .unwrap_or((5, 5));
        
    println!("Spawning NPC at position: ({}, {})", npc_pos.0, npc_pos.1);

    // Spawn NPC
    println!("Creating NPC entity with sprite index 16 at world position: ({}, {})",
        npc_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
        npc_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0));
        
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: game_assets.character_sheet.clone(),
            sprite: TextureAtlasSprite {
                index: 16, // Position 1.a in rogues tileset (sprite at 1,0)
                ..default()
            },
            transform: Transform::from_xyz(
                npc_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                npc_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                1.0
            ),
            ..default()
        },
        Npc::default(),
        Position::new(npc_pos.0, npc_pos.1),
    ));

    // Find valid floor tiles for NPC spawn
    let floor_tiles: Vec<(i32, i32)> = (0..MAP_WIDTH as usize * MAP_HEIGHT as usize)
        .filter(|&i| {
            let row = i / MAP_WIDTH as usize;
            let col = i % MAP_WIDTH as usize;
            map.tiles[row][col] == TileType::Floor
        })
        .map(|i| (
            (i % MAP_WIDTH as usize) as i32,
            (i / MAP_WIDTH as usize) as i32
        ))
        .collect();
        
    println!("Found {} floor tiles for NPC spawning", floor_tiles.len());

    // Choose random position away from player spawn
    let spawn_pos = map.get_spawn_position();
    let npc_pos = floor_tiles.into_iter()
        .filter(|pos| {
            let dx = (pos.0 - spawn_pos.0 as i32).abs();
            let dy = (pos.1 - spawn_pos.1 as i32).abs();
            dx + dy > 5 // Minimum Manhattan distance from player
        })
        .collect::<Vec<_>>();
        
    println!("Found {} valid positions for NPC (minimum 5 tiles from player)", npc_pos.len());

    let npc_pos = npc_pos
        .choose(&mut rand::thread_rng())
        .copied()
        .unwrap_or((5, 5));
        
    println!("Spawning NPC at position: ({}, {})", npc_pos.0, npc_pos.1);

    // Spawn NPC
    println!("Creating NPC entity with sprite index 116 at world position: ({}, {})",
        npc_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
        npc_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0));
        
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: game_assets.character_sheet.clone(),
            sprite: TextureAtlasSprite {
                index: 116, // Position 8.e in rogues tileset
                ..default()
            },
            transform: Transform::from_xyz(
                npc_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                npc_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                1.0
            ),
            ..default()
        },
        Npc::default(),
        Position::new(npc_pos.0, npc_pos.1),
    ));

    // Spawn player
    let spawn_pos = map.get_spawn_position();
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: game_assets.character_sheet.clone(),
            sprite: TextureAtlasSprite {
                index: 4 * 16 + 1, // Position 5.b for wizard
                ..default()
            },
            transform: Transform::from_xyz(
                spawn_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                spawn_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                1.0
            ).with_scale(Vec3::splat(1.0)),
            ..default()
        },
        Player,
        Position::new(spawn_pos.0 as i32, spawn_pos.1 as i32),
        PlayerVisibility::default(),
    ));
}

fn update_sprite_positions(
    mut query: Query<(&Position, &mut Transform), With<Player>>,
) {
    for (pos, mut transform) in &mut query {
        transform.translation.x = pos.x as f32 * TILE_SIZE + (TILE_SIZE / 2.0);
        transform.translation.y = pos.y as f32 * TILE_SIZE + (TILE_SIZE / 2.0); 
        transform.translation.z = 1.0;  // Keep player above tiles
    }
}

#[derive(Event)]
struct RegenerateMapEvent;

fn regenerate_map_system(
    mut commands: Commands,
    input_state: Res<InputState>,
    mut events: EventWriter<RegenerateMapEvent>,
) {
    if !input_state.regenerate_map {
        return;
    }
    
    let new_map = TileMap::new();
    commands.insert_resource(new_map);
    events.send(RegenerateMapEvent);
}

fn update_camera_zoom(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<(&mut CameraControl, &mut OrthographicProjection, &mut Transform)>,
    player_query: Query<&Transform, (With<Player>, Without<CameraControl>)>,
) {
    let (mut control, mut projection, mut camera_transform) = camera_query.single_mut();

    // Get player position
    if let Ok(player_transform) = player_query.get_single() {
        let player_pos = player_transform.translation;
        
        // Calculate target camera position
        let target_camera_pos = Vec3::new(
            player_pos.x,
            player_pos.y,
            camera_transform.translation.z
        );
        
        // Calculate how much to interpolate based on zoom
        // When zoomed in (small scale), follow player completely
        // When zoomed out (large scale), allow free movement within bounds
        let follow_weight = (1.0 - control.current_zoom).clamp(0.0, 1.0);
        
        // Interpolate camera position
        camera_transform.translation = camera_transform.translation.lerp(
            target_camera_pos,
            follow_weight * time.delta_seconds() * 5.0
        );
        
        // Apply map boundaries based on zoom level
        let half_width = (MAP_WIDTH as f32 * TILE_SIZE * control.current_zoom) / 2.0;
        let half_height = (MAP_HEIGHT as f32 * TILE_SIZE * control.current_zoom) / 2.0;
        
        // Calculate bounds
        let min_x = half_width;
        let max_x = MAP_WIDTH as f32 * TILE_SIZE - half_width;
        let min_y = half_height;
        let max_y = MAP_HEIGHT as f32 * TILE_SIZE - half_height;
        
        // Clamp camera position within bounds
        camera_transform.translation.x = camera_transform.translation.x.clamp(min_x, max_x);
        camera_transform.translation.y = camera_transform.translation.y.clamp(min_y, max_y);
    }

    // Calculate minimum zoom to fit entire map
    let window_ratio = MAP_WIDTH as f32 / MAP_HEIGHT as f32;
    let min_zoom = if window_ratio > 1.0 {
        1.0 / MAP_WIDTH as f32
    } else {
        1.0 / MAP_HEIGHT as f32
    } * 5.0; // Multiply by 5.0 to ensure the entire map is visible

    // Handle zoom input
    if keyboard.pressed(KeyCode::Plus) || keyboard.pressed(KeyCode::NumpadAdd) || keyboard.pressed(KeyCode::Equals) {
        control.target_zoom = (control.target_zoom - 0.02).max(min_zoom); // Zoom in
    }
    if keyboard.pressed(KeyCode::Minus) || keyboard.pressed(KeyCode::NumpadSubtract) {
        control.target_zoom = (control.target_zoom + 0.02).min(1.0); // Zoom out
    }
    
    // Smoothly interpolate current zoom to target
    let zoom_delta = control.target_zoom - control.current_zoom;
    if zoom_delta.abs() > 0.001 {
        control.current_zoom += zoom_delta * control.zoom_speed * time.delta_seconds();
        
        // Update camera projection
        projection.scale = control.current_zoom;
    }
}

fn handle_npc_interaction(
    keyboard: Res<Input<KeyCode>>,
    mut npc_query: Query<(&Position, &mut Npc)>,
    player_query: Query<&Position, With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::E) {
        return;
    }

    let player_pos = if let Ok(pos) = player_query.get_single() {
        pos
    } else {
        return;
    };

    for (npc_pos, mut npc) in npc_query.iter_mut() {
        let dx = (npc_pos.x - player_pos.x).abs();
        let dy = (npc_pos.y - player_pos.y).abs();
        
        if dx <= 1 && dy <= 1 {
            if !npc.speaking {
                npc.speaking = true;  // Only toggle on if not speaking
            } else {
                npc.speaking = false;  // Only toggle off if already speaking
            }
        }
    }
}

fn render_dialog_boxes(
    mut commands: Commands,
    npc_query: Query<(&Transform, &Npc), Changed<Npc>>, 
    dialog_query: Query<Entity, With<DialogBox>>,
) {
    // Remove any existing dialog boxes
    for entity in dialog_query.iter() {
        commands.entity(entity).despawn();
    }

    // Create new dialog boxes for speaking NPCs
    for (transform, npc) in npc_query.iter() {
        if npc.speaking {
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        npc.dialog_text.clone(),
                        TextStyle {
                            font_size: 16.0,
                            ..default()
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    transform: Transform::from_translation(
                        transform.translation + Vec3::new(0.0, 32.0, 10.0)
                    ),
                    ..default()
                },
                DialogBox::default()
            ));
        }
    }
}
