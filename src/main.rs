use bevy::prelude::*;
use bevy::window::{WindowMode, WindowPosition, MonitorSelection};
use bevy::render::camera::ScalingMode;
use bevy::sprite::{TextureAtlas, TextureAtlasSprite};
use rand::seq::SliceRandom;
use crate::components::{Position, Player, Npc, Tile, DialogBox};
use crate::map::{TileMap, TileType, MAP_WIDTH, MAP_HEIGHT, GridLine, TileEntities, generate_map_visuals, toggle_grid_visibility, update_tile_visibility};
use crate::input::InputState;
use crate::visibility::{PlayerVisibility, update_visibility, setup_visibility_map};
use crate::systems::check_dialog_distance;
use crate::assets::{SpriteAssets, TextureAtlases, load_sprite_assets};
use crate::biome::BiomeManager;
use bevy::text::{Text2dBundle, Text, TextStyle, TextAlignment};

mod components;
mod map;
// mod rendering; // Removed as functionality has been moved to map.rs
mod input;
mod ui;
mod visibility;
mod systems;
mod assets;
mod biome;

// Use the TILE_SIZE from the input module
use crate::input::TILE_SIZE;

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

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    InGame,
}

// GameAssets struct has been replaced by the new asset management system in the assets module

// Add a resource to track dungeon levels
#[derive(Resource)]
pub struct DungeonState {
    pub levels: Vec<TileMap>,
    pub current_level_index: usize,
}

impl Default for DungeonState {
    fn default() -> Self {
        let initial_map = TileMap::new();
        Self {
            levels: vec![initial_map],
            current_level_index: 0,
        }
    }
}

// Add a component for UI prompts
#[derive(Component)]
struct StairPrompt;

// Add a component for the fade effect
#[derive(Component)]
struct FadeEffect {
    timer: Timer,
    fade_in: bool,
    target_level: Option<usize>,
}

#[derive(Event)]
struct RegenerateMapEvent;

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
        .init_resource::<TileEntities>()
        .init_resource::<BiomeManager>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::InGame), (
            initialize_biome_manager,
            spawn_game_world.after(initialize_biome_manager),
            // setup_visibility_map.after(spawn_game_world) // Commented out visibility system
        ))
        .add_systems(
            Update,
            (
                crate::input::handle_input,
                update_camera_zoom,
                update_sprite_positions.after(crate::input::handle_input),
                // update_visibility.after(crate::input::move_player), // Commented out visibility system
                crate::input::move_player.after(crate::input::handle_input),
                check_dialog_distance.after(crate::input::move_player),
                // update_tile_visibility.after(update_visibility), // Commented out visibility system
                handle_npc_interaction,
                render_dialog_boxes,
                regenerate_map_system.after(crate::input::handle_input),
                toggle_grid_visibility,
                handle_map_regeneration
                    .after(regenerate_map_system)
                    .run_if(resource_exists::<TileMap>())
                    .run_if(on_event::<RegenerateMapEvent>()),
                handle_stairs_system.after(crate::input::handle_input),
                update_fade_effects,
                show_stair_prompt,
            ).run_if(in_state(GameState::InGame))
        )
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
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
    
    // Load all sprite assets
    if let Err(e) = load_sprite_assets(&mut commands, asset_server, texture_atlases) {
        eprintln!("Error loading sprite assets: {}", e);
    }

    // Create initial TileMap
    commands.insert_resource(map.clone());
    
    // Create DungeonState with the same map
    commands.insert_resource(DungeonState {
        levels: vec![map],
        current_level_index: 0,
    });
    
    // Initialize BiomeManager as a resource
    commands.init_resource::<BiomeManager>();
}

fn spawn_game_world(
    mut commands: Commands,
    texture_atlases: Res<TextureAtlases>,
    sprite_assets: Res<SpriteAssets>,
    map: Res<TileMap>,
    biome_manager: Res<BiomeManager>,
    existing_entities: Query<Entity, Or<(With<Tile>, With<Player>, With<Npc>, With<GridLine>)>>,
) {
    // First, clean up any existing entities
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn();
    }
    
    // Then spawn new tiles and player
    map::spawn_tiles(&mut commands, &map, &texture_atlases, &sprite_assets, Some(&biome_manager));
    
    // Spawn grid lines
    map::spawn_grid_lines(&mut commands);

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
            texture_atlas: texture_atlases.characters.clone(),
            sprite: TextureAtlasSprite {
                index: crate::assets::get_character_sprite(&sprite_assets, "dwarf"),
                ..default()
            },
            transform: Transform::from_xyz(
                npc_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                npc_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                1.0
            ).with_scale(Vec3::splat(1.0)),
            ..default()
        },
        Npc {
            name: "Dwarf".to_string(),
            dialog: vec![
                "Greetings, adventurer! I am a dwarf explorer.".to_string(),
                "These caves are treacherous. Watch your step!".to_string(),
                "I've been mapping these tunnels for years.".to_string(),
            ],
            current_dialog_index: 0,
            speaking: false,
            dialog_text: "Greetings, adventurer! I am a dwarf explorer.".to_string(),
        },
        Position::new(npc_pos.0.try_into().unwrap(), npc_pos.1.try_into().unwrap()),
    ));

    // Spawn player
    let spawn_pos = map.get_spawn_position();
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlases.characters.clone(),
            sprite: TextureAtlasSprite {
                index: crate::assets::get_character_sprite(&sprite_assets, "male wizard"),
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

fn regenerate_map_system(
    mut commands: Commands,
    input_state: Res<InputState>,
    mut events: EventWriter<RegenerateMapEvent>,
    mut dungeon_state: ResMut<DungeonState>,
) {
    // Only proceed if SHIFT+R was pressed
    if !input_state.regenerate_map {
        return;
    }
    
    // Generate a new map
    let new_map = TileMap::new();
    println!("Regenerating map...");
    
    // Update the map resource
    commands.insert_resource(new_map.clone());
    
    // Update the current level in the dungeon state
    let current_index = dungeon_state.current_level_index;
    if let Some(level) = dungeon_state.levels.get_mut(current_index) {
        *level = new_map;
    }
    
    // Send an event to notify other systems
    events.send(RegenerateMapEvent);
    
    // Log that map regeneration was triggered
    println!("Map regeneration triggered with SHIFT+R");
}

// Add a system to respond to the regenerate map event
fn handle_map_regeneration(
    mut commands: Commands,
    texture_atlases: Res<TextureAtlases>,
    sprite_assets: Res<SpriteAssets>,
    asset_server: Res<AssetServer>,
    map: Res<TileMap>,
    biome_manager: Res<BiomeManager>,
    existing_entities: Query<Entity, Or<(With<Tile>, With<Player>, With<Npc>, With<GridLine>)>>,
    mut tile_entities: ResMut<TileEntities>,
    mut ev_regenerate: EventReader<RegenerateMapEvent>,
) {
    // Only proceed if we received a regenerate map event
    if ev_regenerate.read().next().is_none() {
        return;
    }
    
    println!("Regenerating game world with new map");
    
    // First, clean up any existing entities
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // Generate new map visuals - this will clear tile_entities and repopulate it
    generate_map_visuals(
        &mut commands,
        &map,
        &asset_server,
        &sprite_assets,
        &texture_atlases,
        &biome_manager,
        &mut tile_entities
    );
    
    // Spawn player at the new spawn position
    let spawn_pos = map.get_spawn_position();
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlases.characters.clone(),
            sprite: TextureAtlasSprite {
                index: crate::assets::get_character_sprite(&sprite_assets, "male wizard"),
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
    if !floor_tiles.is_empty() {
        let npc_pos = floor_tiles.into_iter()
            .filter(|pos| {
                let dx = (pos.0 - spawn_pos.0 as i32).abs();
                let dy = (pos.1 - spawn_pos.1 as i32).abs();
                dx + dy > 5 // Minimum Manhattan distance from player
            })
            .collect::<Vec<_>>();
            
        println!("Found {} valid positions for NPC (minimum 5 tiles from player)", npc_pos.len());

        if !npc_pos.is_empty() {
            let npc_pos = npc_pos
                .choose(&mut rand::thread_rng())
                .copied()
                .unwrap_or((5, 5));
                
            println!("Spawning NPC at position: ({}, {})", npc_pos.0, npc_pos.1);

            // Spawn NPC
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: texture_atlases.characters.clone(),
                    sprite: TextureAtlasSprite {
                        index: crate::assets::get_character_sprite(&sprite_assets, "dwarf"),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        npc_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                        npc_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                        1.0
                    ).with_scale(Vec3::splat(1.0)),
                    ..default()
                },
                Npc {
                    name: "Dwarf".to_string(),
                    dialog: vec![
                        "Greetings, adventurer! I am a dwarf explorer.".to_string(),
                        "These caves are treacherous. Watch your step!".to_string(),
                        "I've been mapping these tunnels for years.".to_string(),
                    ],
                    current_dialog_index: 0,
                    speaking: false,
                    dialog_text: "Greetings, adventurer! I am a dwarf explorer.".to_string(),
                },
                Position::new(npc_pos.0.try_into().unwrap(), npc_pos.1.try_into().unwrap()),
            ));
        }
    }
    
    println!("Map regeneration complete with {} tile entities", tile_entities.entities.len());
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

// System to initialize the BiomeManager with tile mappings
fn initialize_biome_manager(
    mut biome_manager: ResMut<BiomeManager>,
    sprite_assets: Res<SpriteAssets>,
) {
    biome_manager.initialize_default_tiles(&sprite_assets.tile_sprites);
    println!("Initialized BiomeManager with tile mappings");
}

// Modify the handle_stairs_system to directly spawn a fade effect
fn handle_stairs_system(
    mut commands: Commands,
    dungeon_state: Res<DungeonState>,
    player_query: Query<&Position, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    fade_query: Query<Entity, With<FadeEffect>>,
) {
    // Don't process if a fade effect is already active
    if !fade_query.is_empty() {
        return;
    }

    let player_pos = player_query.single();
    
    // Check if player is on stairs and pressed SHIFT+E
    if keyboard_input.pressed(KeyCode::ShiftLeft) && keyboard_input.just_pressed(KeyCode::E) {
        let current_map = &dungeon_state.levels[dungeon_state.current_level_index];
        let player_pos_usize = (player_pos.x as usize, player_pos.y as usize);
        
        // Check if on down stairs
        if let Some(down_pos) = current_map.down_stairs_pos {
            if player_pos_usize.0 == down_pos.0 && player_pos_usize.1 == down_pos.1 {
                let target_level = dungeon_state.current_level_index + 1;
                // Directly spawn fade effect
                spawn_fade_effect(&mut commands, false, Some(target_level));
                println!("Stair transition DOWN initiated to level {}", target_level);
            }
        }
        
        // Check if on up stairs
        if let Some(up_pos) = current_map.up_stairs_pos {
            if player_pos_usize.0 == up_pos.0 && player_pos_usize.1 == up_pos.1 && dungeon_state.current_level_index > 0 {
                let target_level = dungeon_state.current_level_index - 1;
                // Directly spawn fade effect
                spawn_fade_effect(&mut commands, false, Some(target_level));
                println!("Stair transition UP initiated to level {}", target_level);
            }
        }
    }
}

// System to update fade effects
fn update_fade_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut fade_query: Query<(Entity, &mut FadeEffect, &mut BackgroundColor)>,
    mut dungeon_state: ResMut<DungeonState>,
    mut player_query: Query<(&mut Transform, &mut Position), With<Player>>,
    asset_server: Res<AssetServer>,
    sprite_assets: Res<SpriteAssets>,
    texture_atlases: Res<TextureAtlases>,
    biome_manager: Res<BiomeManager>,
    mut tile_entities: ResMut<TileEntities>,
    existing_entities: Query<Entity, Or<(With<Tile>, With<Npc>, With<GridLine>)>>,
) {
    for (entity, mut fade, mut bg_color) in fade_query.iter_mut() {
        fade.timer.tick(time.delta());
        
        if fade.fade_in {
            // Fading in (becoming transparent)
            let alpha = 1.0 - fade.timer.percent();
            bg_color.0.set_a(alpha);
            
            if fade.timer.finished() {
                commands.entity(entity).despawn_recursive();
            }
        } else {
            // Fading out (becoming opaque)
            let alpha = fade.timer.percent();
            bg_color.0.set_a(alpha);
            
            if fade.timer.finished() {
                // Handle level transition if target level is set
                if let Some(target_level) = fade.target_level {
                    let current_level = dungeon_state.current_level_index;
                    println!("Level transition: current={}, target={}", current_level, target_level);
                    
                    // Check if we need to generate a new level
                    if target_level >= dungeon_state.levels.len() {
                        println!("Generating new level {} (current levels: {})", target_level, dungeon_state.levels.len());
                        let current_map = &dungeon_state.levels[current_level];
                        let new_level = TileMap::new_level(target_level, Some(current_map));
                        dungeon_state.levels.push(new_level);
                    } else {
                        println!("Using existing level {}", target_level);
                    }
                    
                    // Update current level
                    dungeon_state.current_level_index = target_level;
                    let new_map = &dungeon_state.levels[target_level];
                    
                    // First, clean up any existing entities (similar to handle_map_regeneration)
                    for entity in existing_entities.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                    
                    // Generate new map visuals
                    generate_map_visuals(
                        &mut commands,
                        new_map,
                        &asset_server,
                        &sprite_assets,
                        &texture_atlases,
                        &biome_manager,
                        &mut tile_entities,
                    );
                    
                    // Move player to appropriate stairs position and update both Transform and Position
                    let (mut player_transform, mut player_position) = player_query.single_mut();
                    if target_level > current_level {
                        // Going down, so place at up stairs
                        if let Some(up_pos) = new_map.up_stairs_pos {
                            println!("Moving player to up stairs at {:?}", up_pos);
                            player_transform.translation.x = (up_pos.0 as f32) * TILE_SIZE + (TILE_SIZE / 2.0);
                            player_transform.translation.y = (up_pos.1 as f32) * TILE_SIZE + (TILE_SIZE / 2.0);
                            // Update Position component to match
                            player_position.x = up_pos.0 as i32;
                            player_position.y = up_pos.1 as i32;
                        }
                    } else {
                        // Going up, so place at down stairs
                        if let Some(down_pos) = new_map.down_stairs_pos {
                            println!("Moving player to down stairs at {:?}", down_pos);
                            player_transform.translation.x = (down_pos.0 as f32) * TILE_SIZE + (TILE_SIZE / 2.0);
                            player_transform.translation.y = (down_pos.1 as f32) * TILE_SIZE + (TILE_SIZE / 2.0);
                            // Update Position component to match
                            player_position.x = down_pos.0 as i32;
                            player_position.y = down_pos.1 as i32;
                        }
                    }
                    
                    // Find valid floor tiles for NPC spawn (similar to handle_map_regeneration)
                    let mut npc_pos = Vec::new();
                    for y in 0..MAP_HEIGHT {
                        for x in 0..MAP_WIDTH {
                            if new_map.tiles[y][x] == TileType::Floor {
                                // Don't spawn NPCs at player position or stairs
                                let is_player_pos = player_position.x == x as i32 && player_position.y == y as i32;
                                let is_stairs = new_map.down_stairs_pos.map_or(false, |pos| pos.0 == x && pos.1 == y) ||
                                               new_map.up_stairs_pos.map_or(false, |pos| pos.0 == x && pos.1 == y);
                                
                                if !is_player_pos && !is_stairs {
                                    npc_pos.push((x, y));
                                }
                            }
                        }
                    }
                    
                    // Spawn NPC if we found valid positions
                    if !npc_pos.is_empty() {
                        let npc_pos = npc_pos
                            .choose(&mut rand::thread_rng())
                            .copied()
                            .unwrap_or((5, 5));
                            
                        println!("Spawning NPC at position: ({}, {})", npc_pos.0, npc_pos.1);
                        
                        // Spawn NPC
                        commands.spawn((
                            SpriteSheetBundle {
                                texture_atlas: texture_atlases.characters.clone(),
                                sprite: TextureAtlasSprite {
                                    index: crate::assets::get_character_sprite(&sprite_assets, "dwarf"),
                                    ..default()
                                },
                                transform: Transform::from_xyz(
                                    npc_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                                    npc_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                                    1.0
                                ).with_scale(Vec3::splat(1.0)),
                                ..default()
                            },
                            Npc {
                                name: "Dwarf".to_string(),
                                dialog: vec![
                                    "Greetings, adventurer! I am a dwarf explorer.".to_string(),
                                    "These caves are treacherous. Watch your step!".to_string(),
                                    "I've been mapping these tunnels for years.".to_string(),
                                ],
                                current_dialog_index: 0,
                                speaking: false,
                                dialog_text: "Greetings, adventurer! I am a dwarf explorer.".to_string(),
                            },
                            Position::new(npc_pos.0.try_into().unwrap(), npc_pos.1.try_into().unwrap()),
                        ));
                    }
                    
                    println!("Level transition complete with {} tile entities", tile_entities.entities.len());
                }
                
                // Start fade in
                fade.fade_in = true;
                fade.timer.reset();
            }
        }
    }
}

// Helper function to spawn a fade effect
fn spawn_fade_effect(
    commands: &mut Commands,
    fade_in: bool,
    target_level: Option<usize>,
) {
    let initial_alpha = if fade_in { 1.0 } else { 0.0 };
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            z_index: ZIndex::Global(100),
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, initial_alpha)),
            ..default()
        },
        FadeEffect {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            fade_in,
            target_level,
        },
    ));
}

// Add a system to show a prompt when player is on stairs
fn show_stair_prompt(
    mut commands: Commands,
    dungeon_state: Res<DungeonState>,
    player_query: Query<&Position, With<Player>>,
    prompt_query: Query<Entity, With<StairPrompt>>,
    _asset_server: Res<AssetServer>,
) {
    // Remove any existing prompts
    for entity in prompt_query.iter() {
        commands.entity(entity).despawn();
    }

    // Check if player is on stairs
    if let Ok(player_pos) = player_query.get_single() {
        let current_map = &dungeon_state.levels[dungeon_state.current_level_index];
        let player_pos_usize = (player_pos.x as usize, player_pos.y as usize);
        
        let on_stairs = if let Some(down_pos) = current_map.down_stairs_pos {
            if player_pos_usize.0 == down_pos.0 && player_pos_usize.1 == down_pos.1 {
                Some("Press SHIFT+E to descend")
            } else {
                None
            }
        } else {
            None
        };

        let on_stairs = on_stairs.or_else(|| {
            if let Some(up_pos) = current_map.up_stairs_pos {
                if player_pos_usize.0 == up_pos.0 && player_pos_usize.1 == up_pos.1 {
                    Some("Press SHIFT+E to ascend")
                } else {
                    None
                }
            } else {
                None
            }
        });

        // Show prompt if on stairs
        if let Some(prompt_text) = on_stairs {
            commands.spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(10.0),
                        left: Val::Percent(50.0),
                        right: Val::Auto,
                        ..default()
                    },
                    ..default()
                },
                StairPrompt,
            ))
            .with_children(|parent| {
                parent.spawn(
                    TextBundle::from_section(
                        prompt_text,
                        TextStyle {
                            font_size: 20.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                    .with_text_alignment(TextAlignment::Center)
                    .with_background_color(Color::rgba(0.0, 0.0, 0.0, 0.5)),
                );
            });
        }
    }
}
