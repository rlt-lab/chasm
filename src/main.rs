use bevy::prelude::*;
use bevy::window::{WindowMode, WindowPosition, MonitorSelection};
use bevy::render::camera::ScalingMode;
use bevy::sprite::{TextureAtlas, TextureAtlasSprite};
use rand::seq::SliceRandom;
use rand::Rng;
use crate::components::{Position, Player, Npc, Tile, DialogBox};
use crate::map::{TileMap, TileType, MAP_WIDTH, MAP_HEIGHT, GridLine, TileEntities, generate_map_visuals, toggle_grid_visibility, update_tile_visibility};
use crate::input::InputState;
use crate::visibility::{PlayerVisibility, update_visibility, setup_visibility_map};
use crate::systems::check_dialog_distance;
use crate::assets::{SpriteAssets, TextureAtlases, load_sprite_assets};
use crate::biome::{BiomeManager, BiomeType};
use crate::dialogue::{CharacterType, generate_dialogue, generate_biome_dialogue};
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
mod dialogue;

// Use the TILE_SIZE from the input module
use crate::input::TILE_SIZE;

// Camera control component
#[derive(Component)]
struct CameraControl {
    current_zoom: f32,
    target_zoom: f32,
    zoom_speed: f32,
    original_zoom: f32,
    original_position: Vec3,
}

impl Default for CameraControl {
    fn default() -> Self {
        Self {
            current_zoom: 1.0,
            target_zoom: 0.6,
            zoom_speed: 2.0,
            original_zoom: 1.0,
            original_position: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

// Add a component for dialog camera zoom
#[derive(Component)]
struct DialogZoom {
    target_zoom: f32,
    original_zoom: f32,
    zoom_speed: f32,
    focus_position: Vec2,
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

// Add a component for the fade effect
#[derive(Component)]
struct FadeEffect {
    timer: Timer,
    fade_in: bool,
    target_level: Option<usize>,
}

// Add a component for UI prompts
#[derive(Component)]
struct StairPrompt;

#[derive(Event)]
struct RegenerateMapEvent;

// Add a new resource to track animation state
#[derive(Resource, Default)]
pub struct AnimationState {
    pub animation_in_progress: bool,
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
        .init_resource::<TileEntities>()
        .init_resource::<BiomeManager>()
        .init_resource::<AnimationState>()
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
                crate::input::queue_next_movement.after(crate::input::handle_input),
                update_camera_zoom,
                update_sprite_positions.after(crate::input::handle_input),
                // update_visibility.after(crate::input::move_player), // Commented out visibility system
                crate::input::move_player.after(crate::input::handle_input),
                animate_player_movement.after(crate::input::move_player),
                check_dialog_distance.after(crate::input::move_player),
                // update_tile_visibility.after(update_visibility), // Commented out visibility system
                handle_npc_interaction.after(check_dialog_distance),
                animate_speaking_npcs.after(handle_npc_interaction),
                render_dialog_boxes.after(handle_npc_interaction),
                regenerate_map_system.after(crate::input::handle_input),
                toggle_grid_visibility,
                handle_map_regeneration
                    .after(regenerate_map_system)
                    .run_if(resource_exists::<TileMap>())
                    .run_if(on_event::<RegenerateMapEvent>()),
                handle_stairs_system.after(crate::input::handle_input),
                // update_fade_effects, // Temporarily disabled fade effects
            )
            .chain() // Add chain() to ensure systems run in sequence
            .run_if(in_state(GameState::InGame))
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

// Function to spawn an NPC at a given position with random character type
fn spawn_npc(
    commands: &mut Commands,
    texture_atlases: &TextureAtlases,
    sprite_assets: &SpriteAssets,
    npc_pos: (i32, i32),
    biome: &BiomeType,
) {
    let mut rng = rand::thread_rng();
    
    // Get all available character sprites
    let available_sprites = crate::dialogue::get_available_character_sprites();
    
    // Choose a random sprite
    let sprite_name = available_sprites.choose(&mut rng).unwrap_or(&"dwarf".to_string()).clone();
    
    // Get the sprite index
    let sprite_index = crate::assets::get_character_sprite(sprite_assets, &sprite_name);
    
    // Determine character type from sprite name
    let character_type = CharacterType::from_sprite_name(&sprite_name);
    
    // Generate a name based on character type
    let npc_name = character_type.generate_name();
    
    // Generate cryptic dialogue instead of regular dialogue
    let mut dialog = crate::dialogue::generate_cryptic_dialogue();
    
    // Add biome-specific cryptic dialogue
    let biome_dialog = crate::dialogue::generate_biome_cryptic_dialogue(biome);
    dialog.push(biome_dialog);
    
    // Get the first dialogue line as the initial text
    let dialog_text = dialog.first().cloned().unwrap_or_else(|| "The void watches.".to_string());
    
    println!("Spawning NPC '{}' ({:?}) at position: ({}, {})", npc_name, character_type, npc_pos.0, npc_pos.1);
    
    // Spawn the NPC entity
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlases.characters.clone(),
            sprite: TextureAtlasSprite {
                index: sprite_index,
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
            name: npc_name,
            dialog,
            current_dialog_index: 0,
            speaking: false,
            dialog_text,
            character_type,
            animation_timer: Timer::from_seconds(0.15, TimerMode::Repeating), // Faster animation
            original_scale: Vec3::splat(1.0),
            wiggle_direction: 1.0,
            wiggle_amount: 0.1, // Increased wiggle amount
        },
        Position::new(npc_pos.0, npc_pos.1),
    ));
}

// Update the spawn_game_world function to add PlayerAnimation component
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

    // 10% chance to spawn an NPC
    let mut rng = rand::thread_rng();
    if !npc_pos.is_empty() && rng.gen_bool(0.1) {
        let npc_pos = npc_pos
            .choose(&mut rand::thread_rng())
            .copied()
            .unwrap_or((5, 5));
            
        spawn_npc(&mut commands, &texture_atlases, &sprite_assets, npc_pos, &map.get_biome_at(npc_pos.0 as usize, npc_pos.1 as usize));
    }

    // Spawn player
    let spawn_pos = map.get_spawn_position();
    let player_pos = Vec3::new(
        spawn_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
        spawn_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
        10.0  // Increased z-index to ensure player is always on top
    );
    
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlases.characters.clone(),
            sprite: TextureAtlasSprite {
                index: crate::assets::get_character_sprite(&sprite_assets, "male wizard"),
                ..default()
            },
            transform: Transform::from_translation(player_pos).with_scale(Vec3::splat(1.0)),
            ..default()
        },
        Player,
        Position::new(spawn_pos.0 as i32, spawn_pos.1 as i32),
        PlayerVisibility::default(),
        components::PlayerAnimation::default(),
    ));
}

fn update_sprite_positions(
    mut query: Query<(&Position, &mut Transform, Option<&components::PlayerAnimation>), With<Player>>,
) {
    for (pos, mut transform, animation_opt) in &mut query {
        // Only update position directly if not currently animating
        if let Some(anim) = animation_opt {
            if anim.is_moving {
                // Skip position update if animation is in progress
                continue;
            }
        }
        
        // Update position directly if no animation is in progress
        transform.translation.x = pos.x as f32 * TILE_SIZE + (TILE_SIZE / 2.0);
        transform.translation.y = pos.y as f32 * TILE_SIZE + (TILE_SIZE / 2.0); 
        transform.translation.z = 10.0;  // Keep player above all tiles with higher z-index
    }
}

// Modify the handle_stairs_system to directly handle level transitions without fade effects
fn handle_stairs_system(
    mut commands: Commands,
    mut dungeon_state: ResMut<DungeonState>,
    mut player_query: Query<(&mut Transform, &mut Position), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    texture_atlases: Res<TextureAtlases>,
    sprite_assets: Res<SpriteAssets>,
    asset_server: Res<AssetServer>,
    existing_entities: Query<Entity, Or<(With<Tile>, With<Player>, With<Npc>, With<GridLine>)>>,
    mut tile_entities: ResMut<TileEntities>,
    biome_manager: Res<BiomeManager>,
    map: Res<TileMap>,
) {
    // First check if we have a player entity
    if player_query.is_empty() {
        return;
    }

    let (_, player_position) = player_query.single();
    let player_pos_usize = (player_position.x as usize, player_position.y as usize);
    
    // Always print player position and stair positions for debugging
    println!("Player position: ({}, {})", player_position.x, player_position.y);
    if let Some(down_pos) = map.down_stairs_pos {
        println!("DOWN stairs at: ({}, {})", down_pos.0, down_pos.1);
    } else {
        println!("No DOWN stairs in this map");
    }
    if let Some(up_pos) = map.up_stairs_pos {
        println!("UP stairs at: ({}, {})", up_pos.0, up_pos.1);
    } else {
        println!("No UP stairs in this map");
    }
    
    // Check if player is on stairs
    let on_down_stairs = map.down_stairs_pos.map_or(false, |pos| player_pos_usize.0 == pos.0 && player_pos_usize.1 == pos.1);
    let on_up_stairs = map.up_stairs_pos.map_or(false, |pos| player_pos_usize.0 == pos.0 && player_pos_usize.1 == pos.1);
    
    if on_down_stairs {
        println!("Player is on DOWN stairs");
    }
    if on_up_stairs {
        println!("Player is on UP stairs");
    }
    
    // Check if SHIFT+E was pressed
    if keyboard_input.pressed(KeyCode::ShiftLeft) && keyboard_input.just_pressed(KeyCode::E) {
        println!("SHIFT+E pressed");
        
        // Check if on down stairs
        if on_down_stairs {
            let target_level = dungeon_state.current_level_index + 1;
            println!("Stair transition DOWN initiated to level {}", target_level);
            
            // DIRECT TRANSITION WITHOUT FADE
            // Only proceed if the target level is valid
            if target_level >= dungeon_state.levels.len() {
                // Generate a new level if needed
                println!("Generating new level {}", target_level);
                let new_map = TileMap::new_level(target_level, None);
                dungeon_state.levels.push(new_map);
            }
            
            // Clone the map before borrowing dungeon_state as mutable
            let new_map = dungeon_state.levels[target_level].clone();
            
            // Update the current level index
            dungeon_state.current_level_index = target_level;
            println!("Updated current level index to {}", target_level);
            
            // Update the map resource
            commands.insert_resource(new_map.clone());
            
            // Store the player entity for later respawning
            let player_entity = existing_entities.iter()
                .find(|&e| player_query.get(e).is_ok())
                .expect("Player entity not found");
            
            // Clean up existing entities
            for entity in existing_entities.iter() {
                commands.entity(entity).despawn_recursive();
            }
            
            // Generate new map visuals
            generate_map_visuals(
                &mut commands,
                &new_map,
                &asset_server,
                &sprite_assets,
                &texture_atlases,
                &biome_manager,
                &mut tile_entities
            );
            
            // Spawn a new player at the up stairs position
            if let Some(up_pos) = new_map.up_stairs_pos {
                println!("Spawning player at up stairs: {:?}", up_pos);
                commands.spawn((
                    SpriteSheetBundle {
                        texture_atlas: texture_atlases.characters.clone(),
                        sprite: TextureAtlasSprite {
                            index: crate::assets::get_character_sprite(&sprite_assets, "male wizard"),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            up_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                            up_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                            10.0  // Increased z-index to ensure player is always on top
                        ).with_scale(Vec3::splat(1.0)),
                        ..default()
                    },
                    Player,
                    Position::new(up_pos.0 as i32, up_pos.1 as i32),
                    PlayerVisibility::default(),
                    components::PlayerAnimation::default(),
                ));
            } else {
                println!("WARNING: No up stairs found in the new map!");
                // Fallback to spawn position
                let spawn_pos = new_map.get_spawn_position();
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
                            10.0  // Increased z-index to ensure player is always on top
                        ).with_scale(Vec3::splat(1.0)),
                        ..default()
                    },
                    Player,
                    Position::new(spawn_pos.0 as i32, spawn_pos.1 as i32),
                    PlayerVisibility::default(),
                    components::PlayerAnimation::default(),
                ));
            }
        }
        
        // Check if on up stairs
        if on_up_stairs && dungeon_state.current_level_index > 0 {
            let target_level = dungeon_state.current_level_index - 1;
            println!("Stair transition UP initiated to level {}", target_level);
            
            // DIRECT TRANSITION WITHOUT FADE
            // Clone the map before borrowing dungeon_state as mutable
            let new_map = dungeon_state.levels[target_level].clone();
            
            // Update the current level index
            dungeon_state.current_level_index = target_level;
            println!("Updated current level index to {}", target_level);
            
            // Update the map resource
            commands.insert_resource(new_map.clone());
            
            // Store the player entity for later respawning
            let player_entity = existing_entities.iter()
                .find(|&e| player_query.get(e).is_ok())
                .expect("Player entity not found");
            
            // Clean up existing entities
            for entity in existing_entities.iter() {
                commands.entity(entity).despawn_recursive();
            }
            
            // Generate new map visuals
            generate_map_visuals(
                &mut commands,
                &new_map,
                &asset_server,
                &sprite_assets,
                &texture_atlases,
                &biome_manager,
                &mut tile_entities
            );
            
            // Spawn a new player at the down stairs position
            if let Some(down_pos) = new_map.down_stairs_pos {
                println!("Spawning player at down stairs: {:?}", down_pos);
                commands.spawn((
                    SpriteSheetBundle {
                        texture_atlas: texture_atlases.characters.clone(),
                        sprite: TextureAtlasSprite {
                            index: crate::assets::get_character_sprite(&sprite_assets, "male wizard"),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            down_pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                            down_pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                            10.0  // Increased z-index to ensure player is always on top
                        ).with_scale(Vec3::splat(1.0)),
                        ..default()
                    },
                    Player,
                    Position::new(down_pos.0 as i32, down_pos.1 as i32),
                    PlayerVisibility::default(),
                    components::PlayerAnimation::default(),
                ));
            } else {
                println!("WARNING: No down stairs found in the new map!");
                // Fallback to spawn position
                let spawn_pos = new_map.get_spawn_position();
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
                            10.0  // Increased z-index to ensure player is always on top
                        ).with_scale(Vec3::splat(1.0)),
                        ..default()
                    },
                    Player,
                    Position::new(spawn_pos.0 as i32, spawn_pos.1 as i32),
                    PlayerVisibility::default(),
                    components::PlayerAnimation::default(),
                ));
            }
        }
    }
}

// Modify regenerate_map_system to directly handle map regeneration without fade effects
fn regenerate_map_system(
    mut commands: Commands,
    input_state: Res<InputState>,
    mut dungeon_state: ResMut<DungeonState>,
    mut player_query: Query<(&mut Transform, &mut Position), With<Player>>,
    texture_atlases: Res<TextureAtlases>,
    sprite_assets: Res<SpriteAssets>,
    asset_server: Res<AssetServer>,
    existing_entities: Query<Entity, Or<(With<Tile>, With<Player>, With<Npc>, With<GridLine>)>>,
    mut tile_entities: ResMut<TileEntities>,
    biome_manager: Res<BiomeManager>,
    mut events: EventWriter<RegenerateMapEvent>,
) {
    // Only proceed if SHIFT+R was pressed
    if !input_state.regenerate_map {
        return;
    }
    
    // First check if we have a player entity
    if player_query.is_empty() {
        return;
    }
    
    println!("Map regeneration triggered with SHIFT+R");
    
    // Get the current level index
    let current_index = dungeon_state.current_level_index;
    
    // DIRECT REGENERATION WITHOUT FADE
    // Generate a new map with the same level index
    println!("Regenerating map for level {}", current_index);
    let new_map = TileMap::new_level(current_index, None);
    
    // Update the map in dungeon state
    if let Some(level) = dungeon_state.levels.get_mut(current_index) {
        *level = new_map.clone();
    }
    
    // Update the map resource
    commands.insert_resource(new_map.clone());
    
    // Send an event to notify other systems
    events.send(RegenerateMapEvent);
    
    // Store the player entity for later respawning
    let player_entity = existing_entities.iter()
        .find(|&e| player_query.get(e).is_ok())
        .expect("Player entity not found");
    
    // Clean up existing entities
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // Generate new map visuals
    generate_map_visuals(
        &mut commands,
        &new_map,
        &asset_server,
        &sprite_assets,
        &texture_atlases,
        &biome_manager,
        &mut tile_entities
    );
    
    // Spawn a new player at the spawn position
    let spawn_pos = new_map.get_spawn_position();
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
                10.0  // Increased z-index to ensure player is always on top
            ).with_scale(Vec3::splat(1.0)),
            ..default()
        },
        Player,
        Position::new(spawn_pos.0 as i32, spawn_pos.1 as i32),
        PlayerVisibility::default(),
        components::PlayerAnimation::default(),
    ));
    
    println!("Player spawned at position: {:?}", spawn_pos);
    
    // Find valid floor tiles for NPC spawn
    let mut npc_pos = Vec::new();
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            if new_map.tiles[y][x] == TileType::Floor {
                // Check if this is the player position or stairs
                let is_player_pos = spawn_pos.0 == x && spawn_pos.1 == y;
                let is_stairs = new_map.down_stairs_pos.map_or(false, |pos| pos.0 == x && pos.1 == y) ||
                               new_map.up_stairs_pos.map_or(false, |pos| pos.0 == x && pos.1 == y);
                
                if !is_player_pos && !is_stairs {
                    npc_pos.push((x as i32, y as i32));
                }
            }
        }
    }
    
    // Spawn NPC if we found valid positions with 10% chance
    let mut rng = rand::thread_rng();
    if !npc_pos.is_empty() && rng.gen_bool(0.1) {
        let npc_pos = npc_pos
            .choose(&mut rand::thread_rng())
            .copied()
            .unwrap_or((5, 5));
        
        println!("Spawning NPC at position: ({}, {})", npc_pos.0, npc_pos.1);
        
        // Spawn NPC
        spawn_npc(&mut commands, &texture_atlases, &sprite_assets, npc_pos, &new_map.get_biome_at(npc_pos.0 as usize, npc_pos.1 as usize));
    }
    
    println!("Map regeneration completed");
}

fn update_camera_zoom(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<(&mut CameraControl, &mut OrthographicProjection, &mut Transform), Without<Player>>,
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
    mut params: ParamSet<(
        Query<(Entity, &Position, &mut Npc, &Transform)>,
        Query<(&Position, &Transform), With<Player>>,
        Query<(&mut CameraControl, &mut Transform), Without<Player>>
    )>,
) {
    if !keyboard.just_pressed(KeyCode::E) {
        return;
    }

    // First, collect all the data we need
    let player_data = if let Ok(pos) = params.p1().get_single() {
        Some((Position { x: pos.0.x, y: pos.0.y }, pos.1.translation))
    } else {
        None
    };
    
    if player_data.is_none() {
        return;
    }
    
    let (player_pos, player_transform_translation) = player_data.unwrap();
    
    // Find NPCs that are close to the player
    let mut npc_to_interact = None;
    
    for (entity_id, npc_pos, npc, npc_transform) in params.p0().iter() {
        let dx = (npc_pos.x - player_pos.x).abs();
        let dy = (npc_pos.y - player_pos.y).abs();
        
        if dx <= 1 && dy <= 1 {
            // Found an NPC to interact with
            let next_dialog_index = (npc.current_dialog_index + 1) % npc.dialog.len();
            let next_dialog = npc.dialog[next_dialog_index].clone();
            
            npc_to_interact = Some((
                entity_id,
                npc.speaking,
                next_dialog,
                npc_transform.translation,
                npc_transform.scale,
                npc.current_dialog_index
            ));
            break;
        }
    }
    
    // If we found an NPC to interact with, update it and the camera
    if let Some((entity_id, is_speaking, next_dialog, npc_translation, npc_scale, current_index)) = npc_to_interact {
        // First update the camera
        {
            let mut camera_query = params.p2();
            let (mut camera_control, mut camera_transform) = camera_query.single_mut();
            
            // Calculate midpoint between player and NPC for camera focus
            let midpoint = Vec3::new(
                (player_transform_translation.x + npc_translation.x) / 2.0,
                (player_transform_translation.y + npc_translation.y) / 2.0,
                camera_transform.translation.z
            );
            
            if !is_speaking {
                // Store original camera zoom and position
                camera_control.original_zoom = camera_control.current_zoom;
                camera_control.original_position = camera_transform.translation;
                
                // Set target zoom for close-up
                camera_control.target_zoom = 0.2; // Closer zoom for dialog
                
                // Increase zoom speed for faster transition
                camera_control.zoom_speed = 5.0; // Faster zoom speed
                
                // Set camera position to focus on the conversation
                camera_transform.translation = midpoint;
            } else {
                // Reset camera zoom to previous level
                camera_control.target_zoom = camera_control.original_zoom;
                
                // Return to original position
                camera_transform.translation = camera_control.original_position;
                
                // Reset zoom speed to normal
                camera_control.zoom_speed = 2.0; // Normal zoom speed
            }
        }
        
        // Then update the NPC
        {
            let mut npc_query = params.p0();
            if let Ok((_, _, mut npc, _)) = npc_query.get_mut(entity_id) {
                if !is_speaking {
                    // Start speaking
                    npc.speaking = true;
                    
                    // Advance to the next dialog line
                    npc.current_dialog_index = (current_index + 1) % npc.dialog.len();
                    npc.dialog_text = next_dialog;
                    
                    // Store original scale for animation
                    npc.original_scale = npc_scale;
                } else {
                    // Stop speaking
                    npc.speaking = false;
                }
            }
        }
    }
}

// Add a system to animate speaking NPCs with side-to-side wiggle
fn animate_speaking_npcs(
    time: Res<Time>,
    mut query: Query<(&mut Npc, &mut Transform)>,
) {
    for (mut npc, mut transform) in query.iter_mut() {
        if npc.speaking {
            // Update the animation timer
            npc.animation_timer.tick(time.delta());
            
            // Wiggle the sprite with a more pronounced rotation when the timer finishes
            if npc.animation_timer.just_finished() {
                // Change wiggle direction
                npc.wiggle_direction *= -1.0;
                
                // Apply wiggle as a more pronounced rotation (convert to radians)
                let wiggle_angle = npc.wiggle_amount * npc.wiggle_direction * 0.4; // Increased amount for rotation
                transform.rotation = Quat::from_rotation_z(wiggle_angle);
            }
        } else if transform.rotation != Quat::IDENTITY {
            // Reset rotation when not speaking
            transform.rotation = Quat::IDENTITY;
        }
    }
}

fn render_dialog_boxes(
    mut commands: Commands,
    npc_query: Query<(Entity, &Transform, &Npc)>,
    dialog_query: Query<Entity, With<DialogBox>>,
    asset_server: Res<AssetServer>,
) {
    // Remove any existing dialog boxes
    for entity in dialog_query.iter() {
        commands.entity(entity).despawn();
    }

    // Create new dialog boxes for speaking NPCs
    for (_entity, transform, npc) in npc_query.iter() {
        if npc.speaking {
            // Calculate the width based on text length (with min and max bounds)
            let text_length = npc.dialog_text.len() as f32;
            let char_width = 5.5; // Approximate width per character in pixels
            let min_width = 3.0 * TILE_SIZE;
            let max_width = 6.0 * TILE_SIZE;
            let width = (text_length * char_width).clamp(min_width, max_width);
            
            // Create a background for the dialog box - dark gray with transparency
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgba(0.2, 0.2, 0.2, 0.85), // Dark gray with transparency
                        custom_size: Some(Vec2::new(width, 30.0)), // Even smaller height
                        ..default()
                    },
                    transform: Transform::from_translation(
                        transform.translation + Vec3::new(0.0, 35.0, 5.0) // Positioned just above NPC
                    ),
                    ..default()
                },
                DialogBox {
                    text: npc.dialog_text.clone(),
                    visible: true,
                },
            ));
            
            // Create the text - adjusted for the smaller box, without character name
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        npc.dialog_text.clone(),
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Light.ttf"),
                            font_size: 10.0, // Even smaller font
                            color: Color::WHITE, // White text
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    transform: Transform::from_translation(
                        transform.translation + Vec3::new(0.0, 35.0, 10.0) // Positioned just above NPC
                    ),
                    ..default()
                },
                DialogBox {
                    text: npc.dialog_text.clone(),
                    visible: true,
                },
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

// System to update fade effects
fn update_fade_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut fade_query: Query<(Entity, &mut FadeEffect, &mut BackgroundColor)>,
    mut dungeon_state: ResMut<DungeonState>,
    texture_atlases: Res<TextureAtlases>,
    sprite_assets: Res<SpriteAssets>,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(&mut Transform, &mut Position), With<Player>>,
    existing_entities: Query<Entity, Or<(With<Tile>, With<Npc>, With<GridLine>)>>,
    mut tile_entities: ResMut<TileEntities>,
    biome_manager: Res<BiomeManager>,
    mut events: EventWriter<RegenerateMapEvent>,
) {
    // Debug: Print the number of fade effects
    if !fade_query.is_empty() {
        println!("Processing {} fade effects", fade_query.iter().count());
    }

    for (entity, mut fade, mut background) in fade_query.iter_mut() {
        // Update fade timer
        fade.timer.tick(time.delta());
        
        // Calculate alpha based on fade direction and progress
        let progress = fade.timer.percent();
        let alpha = if fade.fade_in {
            progress // Fade in: 0.0 -> 1.0
        } else {
            1.0 - progress // Fade out: 1.0 -> 0.0
        };
        
        // Update background alpha
        background.0.set_a(alpha);
        
        // Debug: Print fade progress
        println!("Fade progress: {:.2}, Alpha: {:.2}, Fade in: {}, Target level: {:?}", 
                 progress, alpha, fade.fade_in, fade.target_level);
        
        // Check if fade is complete
        if fade.timer.finished() {
            println!("Fade effect completed!");
            
            // If this was a fade out, handle the transition
            if !fade.fade_in && fade.target_level.is_some() {
                let target_level = fade.target_level.unwrap();
                println!("Transitioning to level {}", target_level);
                
                // Get the current level index
                let current_level = dungeon_state.current_level_index;
                
                // Check if this is a map regeneration (same level)
                let is_regeneration = target_level == current_level;
                
                if is_regeneration {
                    // Generate a new map with the same level index
                    println!("Regenerating map for level {}", target_level);
                    let new_map = TileMap::new_level(target_level, None);
                    
                    // Update the map in dungeon state
                    if let Some(level) = dungeon_state.levels.get_mut(target_level) {
                        *level = new_map.clone();
                    }
                    
                    // Update the map resource
                    commands.insert_resource(new_map.clone());
                    
                    // Send an event to notify other systems
                    events.send(RegenerateMapEvent);
                    
                    // Clean up existing entities
                    for entity in existing_entities.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                    
                    // Generate new map visuals
                    generate_map_visuals(
                        &mut commands,
                        &new_map,
                        &asset_server,
                        &sprite_assets,
                        &texture_atlases,
                        &biome_manager,
                        &mut tile_entities
                    );
                    
                    // Move player to spawn position
                    let spawn_pos = new_map.get_spawn_position();
                    let (mut player_transform, mut player_position) = player_query.single_mut();
                    player_transform.translation.x = (spawn_pos.0 as f32) * TILE_SIZE + (TILE_SIZE / 2.0);
                    player_transform.translation.y = (spawn_pos.1 as f32) * TILE_SIZE + (TILE_SIZE / 2.0);
                    player_position.x = spawn_pos.0 as i32;
                    player_position.y = spawn_pos.1 as i32;
                    
                    println!("Player moved to spawn position: {:?}", spawn_pos);
                } else {
                    // Only proceed if the target level is valid
                    if target_level >= dungeon_state.levels.len() {
                        // Generate a new level if needed
                        println!("Generating new level {}", target_level);
                        let new_map = TileMap::new_level(target_level, None);
                        dungeon_state.levels.push(new_map);
                    }
                    
                    // Clone the map before borrowing dungeon_state as mutable
                    let new_map = dungeon_state.levels[target_level].clone();
                    
                    // Update the current level index
                    dungeon_state.current_level_index = target_level;
                    println!("Updated current level index to {}", target_level);
                    
                    // Update the map resource
                    commands.insert_resource(new_map.clone());
                    
                    // Clean up existing entities
                    for entity in existing_entities.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                    
                    // Generate new map visuals
                    generate_map_visuals(
                        &mut commands,
                        &new_map,
                        &asset_server,
                        &sprite_assets,
                        &texture_atlases,
                        &biome_manager,
                        &mut tile_entities
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
                        } else {
                            println!("WARNING: No up stairs found in the new map!");
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
                        } else {
                            println!("WARNING: No down stairs found in the new map!");
                        }
                    }
                }
                
                // Find valid floor tiles for NPC spawn (similar to handle_map_regeneration)
                let mut npc_pos = Vec::new();
                for y in 0..MAP_HEIGHT {
                    for x in 0..MAP_WIDTH {
                        let map = if is_regeneration {
                            // Use the newly generated map for regeneration
                            dungeon_state.levels[target_level].clone()
                        } else {
                            // Use the map from the target level
                            dungeon_state.levels[target_level].clone()
                        };
                        
                        if map.tiles[y][x] == TileType::Floor {
                            // Get player position
                            let (_, player_position) = player_query.single();
                            
                            // Don't spawn NPCs at player position or stairs
                            let is_player_pos = player_position.x == x as i32 && player_position.y == y as i32;
                            let is_stairs = map.down_stairs_pos.map_or(false, |pos| pos.0 == x && pos.1 == y) ||
                                           map.up_stairs_pos.map_or(false, |pos| pos.0 == x && pos.1 == y);
                            
                            if !is_player_pos && !is_stairs {
                                npc_pos.push((x as i32, y as i32));
                            }
                        }
                    }
                }
                
                // Spawn NPC if we found valid positions with 10% chance
                let mut rng = rand::thread_rng();
                if !npc_pos.is_empty() && rng.gen_bool(0.1) {
                    let npc_pos = npc_pos
                        .choose(&mut rand::thread_rng())
                        .copied()
                        .unwrap_or((5, 5));
                    
                    // Get the map for biome information
                    let map = dungeon_state.levels[target_level].clone();
                    
                    println!("Spawning NPC at position: ({}, {})", npc_pos.0, npc_pos.1);
                    
                    // Spawn NPC
                    spawn_npc(&mut commands, &texture_atlases, &sprite_assets, npc_pos, &map.get_biome_at(npc_pos.0 as usize, npc_pos.1 as usize));
                }
                
                // Start fade in
                spawn_fade_effect(&mut commands, true, None);
            } else {
                // Remove the fade effect entity
                commands.entity(entity).despawn();
                println!("Removed fade effect entity");
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
    
    // First, ensure we're creating a proper UI element with a background color
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
    
    // Log the fade effect creation for debugging
    if fade_in {
        println!("Created fade IN effect");
    } else {
        println!("Created fade OUT effect with target level: {:?}", target_level);
    }
}

// Update the handle_map_regeneration function to handle map regeneration events
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
    
    println!("Handling map regeneration event");
    
    // The actual regeneration logic is now handled in regenerate_map_system
    // This function is kept for compatibility with the existing event system
}

// Add a new system to animate player movement with hop and wobble
fn animate_player_movement(
    time: Res<Time>,
    input_state: Res<InputState>,
    mut player_query: Query<(Entity, &Position, &mut Transform, &mut components::PlayerAnimation), With<Player>>,
    mut commands: Commands,
    map: Res<TileMap>,
    mut animation_state: ResMut<AnimationState>,
) {
    for (entity, position, mut transform, mut animation) in player_query.iter_mut() {
        // If currently animating, continue the animation
        if animation.is_moving {
            // Ensure animation state is marked as in progress
            animation_state.animation_in_progress = true;
            
            // Update the timer
            animation.animation_timer.tick(time.delta());
            
            // Calculate progress (0.0 to 1.0)
            let progress = animation.animation_timer.percent();
            
            // Calculate the current position with a hop
            // Use a sine curve for the hop (peaks at 0.5 progress)
            let hop_offset = (progress * std::f32::consts::PI).sin() * animation.hop_height;
            
            // Interpolate between start and target positions
            let current_pos = animation.start_pos.lerp(animation.target_pos, progress);
            
            // Apply the hop offset to the y coordinate
            transform.translation = Vec3::new(
                current_pos.x,
                current_pos.y + hop_offset,
                current_pos.z
            );
            
            // Apply wobble (rotation) based on progress
            // Maximum wobble at the middle of the animation
            let wobble_factor = (progress * std::f32::consts::PI).sin();
            let wobble_angle = animation.wobble_direction * animation.wobble_amount * wobble_factor;
            transform.rotation = Quat::from_rotation_z(wobble_angle);
            
            // Check if animation is complete
            if animation.animation_timer.finished() {
                // Reset animation state
                animation.is_moving = false;
                animation_state.animation_in_progress = false;
                
                // Ensure the sprite is at exactly the target position with no rotation
                transform.translation = animation.target_pos;
                transform.rotation = Quat::IDENTITY;
                
                println!("Animation complete, final position: {:?}", transform.translation);
                
                // Check if we have a queued direction to process
                if animation.queued_direction.is_some() {
                    let direction = animation.queued_direction.unwrap();
                    let mut new_pos_x = position.x;
                    let mut new_pos_y = position.y;
                    
                    // Calculate new position based on queued direction
                    match direction {
                        components::MovementDirection::Up => new_pos_y += 1,
                        components::MovementDirection::Down => new_pos_y -= 1,
                        components::MovementDirection::Left => new_pos_x -= 1,
                        components::MovementDirection::Right => new_pos_x += 1,
                    }
                    
                    // Check if the new position is valid
                    if new_pos_x >= 0 && new_pos_x < crate::map::MAP_WIDTH as i32 &&
                       new_pos_y >= 0 && new_pos_y < crate::map::MAP_HEIGHT as i32 {
                        let tile_type = map.tiles[new_pos_y as usize][new_pos_x as usize];
                        if tile_type != TileType::Wall {
                            // Create a new Position component
                            let new_pos = Position::new(new_pos_x, new_pos_y);
                            
                            // Update the player's position component
                            commands.entity(entity).insert(new_pos);
                            
                            // Start a new animation immediately
                            let target_pos = Vec3::new(
                                new_pos_x as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                                new_pos_y as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                                10.0  // Keep z-coordinate at 10.0 to ensure player is always on top
                            );
                            
                            animation.start_pos = transform.translation;
                            animation.target_pos = target_pos;
                            animation.is_moving = true;
                            animation_state.animation_in_progress = true;
                            
                            // Store the movement direction
                            animation.last_movement_direction = Some(direction);
                            
                            // Clear the queued direction
                            animation.queued_direction = None;
                            
                            // Use consistent animation duration
                            let animation_duration = 0.2;
                            animation.animation_timer = Timer::from_seconds(animation_duration, TimerMode::Once);
                            
                            // Flip the wobble direction for alternating effect
                            animation.wobble_direction *= -1.0;
                            
                            println!("Processing queued movement in direction {:?}, animation speed: {:.2}s", 
                                     direction, animation_duration);
                            
                            // Skip the rest of the processing since we've started a new animation
                            continue;
                        }
                    }
                    
                    // If we couldn't process the queued direction, clear it
                    animation.queued_direction = None;
                }
                
                // Handle continuous movement - start a new movement in the same direction if key is still held
                if input_state.continuous_movement && animation.last_movement_direction.is_some() {
                    let direction = animation.last_movement_direction.unwrap();
                    let mut new_pos_x = position.x;
                    let mut new_pos_y = position.y;
                    
                    // Calculate new position based on direction
                    match direction {
                        components::MovementDirection::Up => new_pos_y += 1,
                        components::MovementDirection::Down => new_pos_y -= 1,
                        components::MovementDirection::Left => new_pos_x -= 1,
                        components::MovementDirection::Right => new_pos_x += 1,
                    }
                    
                    // Check if the new position is valid
                    if new_pos_x >= 0 && new_pos_x < crate::map::MAP_WIDTH as i32 &&
                       new_pos_y >= 0 && new_pos_y < crate::map::MAP_HEIGHT as i32 {
                        let tile_type = map.tiles[new_pos_y as usize][new_pos_x as usize];
                        if tile_type != TileType::Wall {
                            // Create a new Position component
                            let new_pos = Position::new(new_pos_x, new_pos_y);
                            
                            // Update the player's position component
                            commands.entity(entity).insert(new_pos);
                            
                            // Start a new animation immediately
                            let target_pos = Vec3::new(
                                new_pos_x as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                                new_pos_y as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                                10.0  // Keep z-coordinate at 10.0 to ensure player is always on top
                            );
                            
                            animation.start_pos = transform.translation;
                            animation.target_pos = target_pos;
                            animation.is_moving = true;
                            animation_state.animation_in_progress = true;
                            
                            // Use consistent animation duration for continuous movement
                            let animation_duration = 0.2;
                            animation.animation_timer = Timer::from_seconds(animation_duration, TimerMode::Once);
                            
                            // Flip the wobble direction for alternating effect
                            animation.wobble_direction *= -1.0;
                            
                            println!("Continuing movement in direction {:?}, animation speed: {:.2}s", 
                                     direction, animation_duration);
                        }
                    }
                } else {
                    // Reset rapid press count when not continuing movement
                    animation.rapid_press_count = 0;
                }
            }
        }
        // Only start a new animation if not currently animating
        else if (input_state.up || input_state.down || input_state.left || input_state.right) && !animation_state.animation_in_progress {
            // Calculate the target position based on the Position component
            let target_pos = Vec3::new(
                position.x as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                position.y as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                10.0  // Keep z-coordinate at 10.0 to ensure player is always on top
            );
            
            // Only start animation if the position actually changed
            if transform.translation != target_pos {
                // Store the starting position
                animation.start_pos = transform.translation;
                animation.target_pos = target_pos;
                animation.is_moving = true;
                animation_state.animation_in_progress = true;
                
                // Store the movement direction
                if input_state.up {
                    animation.last_movement_direction = Some(components::MovementDirection::Up);
                } else if input_state.down {
                    animation.last_movement_direction = Some(components::MovementDirection::Down);
                } else if input_state.left {
                    animation.last_movement_direction = Some(components::MovementDirection::Left);
                } else if input_state.right {
                    animation.last_movement_direction = Some(components::MovementDirection::Right);
                }
                
                // Check for rapid key presses (within 0.3 seconds)
                let current_time = time.elapsed_seconds_f64();
                if current_time - input_state.last_key_press_time < 0.3 {
                    // Increment rapid press count (max 5) - we still track this but don't use it for speed
                    animation.rapid_press_count = (animation.rapid_press_count + 1).min(5);
                } else {
                    // Reset rapid press count
                    animation.rapid_press_count = 0;
                }
                
                // Use consistent animation duration regardless of rapid press count
                let animation_duration = 0.2;
                animation.animation_timer = Timer::from_seconds(animation_duration, TimerMode::Once);
                
                // Flip the wobble direction for alternating effect
                animation.wobble_direction *= -1.0;
                
                // Print debug info
                println!("Starting animation from {:?} to {:?} with wobble direction {}, speed: {:.2}s, rapid presses: {}", 
                         animation.start_pos, animation.target_pos, animation.wobble_direction, 
                         animation_duration, animation.rapid_press_count);
            }
        }
    }
}
