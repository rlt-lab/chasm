use bevy::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::HashMap;

use crate::biome::BiomeType;
use crate::components::{Animal, AnimalType, Position, AnimalTooltip, GameTurn, AnimalAnimation, MovementDirection, Npc, AnimalNpc};
use crate::input::TILE_SIZE;
use crate::map::{TileMap, TileType, MAP_WIDTH, MAP_HEIGHT};
use crate::AnimationState;
use crate::dialogue::CharacterType;

// Maximum number of animals that can spawn on a map
pub const MAX_ANIMALS_PER_MAP: usize = 3;

// Structure to hold animal spawn data
pub struct AnimalSpawnData {
    pub animal_type: AnimalType,
    pub spawn_rate: f32, // As a percentage (0-100)
    pub sprite_index: usize,
}

// Resource to manage animal spawning
#[derive(Resource)]
pub struct AnimalManager {
    pub biome_animals: HashMap<BiomeType, Vec<AnimalSpawnData>>,
    pub animal_sprites: HashMap<AnimalType, usize>,
}

impl Default for AnimalManager {
    fn default() -> Self {
        Self {
            biome_animals: HashMap::new(),
            animal_sprites: HashMap::new(),
        }
    }
}

impl AnimalManager {
    // Initialize with the animal sprite indices
    pub fn initialize(&mut self, sprite_assets: &HashMap<String, usize>) {
        // Map animal types to sprite indices
        self.register_animal_sprites(sprite_assets);
        
        // Set up biome-specific animal lists with spawn rates
        self.setup_biome_animals();
    }
    
    // Register animal sprites from the sprite assets
    fn register_animal_sprites(&mut self, sprite_assets: &HashMap<String, usize>) {
        // Snakes
        if let Some(&index) = sprite_assets.get("snake") {
            self.animal_sprites.insert(AnimalType::Snake, index);
        }
        if let Some(&index) = sprite_assets.get("cobra") {
            self.animal_sprites.insert(AnimalType::Cobra, index);
        }
        if let Some(&index) = sprite_assets.get("kingsnake") {
            self.animal_sprites.insert(AnimalType::Kingsnake, index);
        }
        if let Some(&index) = sprite_assets.get("black mamba") {
            self.animal_sprites.insert(AnimalType::BlackMamba, index);
        }
        
        // Rodents
        if let Some(&index) = sprite_assets.get("rat") {
            self.animal_sprites.insert(AnimalType::Rat, index);
        }
        
        // Predators
        if let Some(&index) = sprite_assets.get("grizzly bear") {
            self.animal_sprites.insert(AnimalType::GrizzlyBear, index);
        }
        if let Some(&index) = sprite_assets.get("black bear") {
            self.animal_sprites.insert(AnimalType::BlackBear, index);
        }
        if let Some(&index) = sprite_assets.get("honeybadger") {
            self.animal_sprites.insert(AnimalType::Honeybadger, index);
        }
        
        // Canines/Felines
        if let Some(&index) = sprite_assets.get("dog") {
            self.animal_sprites.insert(AnimalType::Dog, index);
        }
        if let Some(&index) = sprite_assets.get("cat") {
            self.animal_sprites.insert(AnimalType::Cat, index);
        }
        
        // Livestock/Wild
        if let Some(&index) = sprite_assets.get("pig") {
            self.animal_sprites.insert(AnimalType::Pig, index);
        }
        if let Some(&index) = sprite_assets.get("boar") {
            self.animal_sprites.insert(AnimalType::Boar, index);
        }
        if let Some(&index) = sprite_assets.get("capybara") {
            self.animal_sprites.insert(AnimalType::Capybara, index);
        }
        if let Some(&index) = sprite_assets.get("beaver") {
            self.animal_sprites.insert(AnimalType::Beaver, index);
        }
        if let Some(&index) = sprite_assets.get("water buffalo") {
            self.animal_sprites.insert(AnimalType::WaterBuffalo, index);
        }
        if let Some(&index) = sprite_assets.get("yak") {
            self.animal_sprites.insert(AnimalType::Yak, index);
        }
        if let Some(&index) = sprite_assets.get("mallard duck") {
            self.animal_sprites.insert(AnimalType::MallardDuck, index);
        }
        if let Some(&index) = sprite_assets.get("sheep (ram)") {
            self.animal_sprites.insert(AnimalType::SheepRam, index);
        }
        if let Some(&index) = sprite_assets.get("sheep (ewe)") {
            self.animal_sprites.insert(AnimalType::SheepEwe, index);
        }
    }
    
    // Set up biome-specific animal lists with spawn rates
    fn setup_biome_animals(&mut self) {
        // Caves biome animals
        let mut caves_animals = Vec::new();
        caves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Snake,
            spawn_rate: 6.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Snake).unwrap_or(&0),
        });
        caves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Cobra,
            spawn_rate: 3.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Cobra).unwrap_or(&0),
        });
        caves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Kingsnake,
            spawn_rate: 3.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Kingsnake).unwrap_or(&0),
        });
        caves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::BlackMamba,
            spawn_rate: 2.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::BlackMamba).unwrap_or(&0),
        });
        caves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Rat,
            spawn_rate: 15.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Rat).unwrap_or(&0),
        });
        caves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Honeybadger,
            spawn_rate: 3.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Honeybadger).unwrap_or(&0),
        });
        caves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::GrizzlyBear,
            spawn_rate: 1.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::GrizzlyBear).unwrap_or(&0),
        });
        caves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::BlackBear,
            spawn_rate: 1.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::BlackBear).unwrap_or(&0),
        });
        caves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Pig,
            spawn_rate: 2.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Pig).unwrap_or(&0),
        });
        caves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Boar,
            spawn_rate: 1.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Boar).unwrap_or(&0),
        });
        self.biome_animals.insert(BiomeType::Caves, caves_animals);
        
        // Labyrinth biome animals
        let mut labyrinth_animals = Vec::new();
        labyrinth_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Snake,
            spawn_rate: 6.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Snake).unwrap_or(&0),
        });
        labyrinth_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Cobra,
            spawn_rate: 3.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Cobra).unwrap_or(&0),
        });
        labyrinth_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Kingsnake,
            spawn_rate: 3.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Kingsnake).unwrap_or(&0),
        });
        labyrinth_animals.push(AnimalSpawnData {
            animal_type: AnimalType::BlackMamba,
            spawn_rate: 2.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::BlackMamba).unwrap_or(&0),
        });
        labyrinth_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Rat,
            spawn_rate: 10.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Rat).unwrap_or(&0),
        });
        labyrinth_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Cat,
            spawn_rate: 5.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Cat).unwrap_or(&0),
        });
        labyrinth_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Dog,
            spawn_rate: 5.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Dog).unwrap_or(&0),
        });
        self.biome_animals.insert(BiomeType::Labyrinth, labyrinth_animals);
        
        // Catacombs biome animals
        let mut catacombs_animals = Vec::new();
        catacombs_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Rat,
            spawn_rate: 20.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Rat).unwrap_or(&0),
        });
        catacombs_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Snake,
            spawn_rate: 6.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Snake).unwrap_or(&0),
        });
        catacombs_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Dog,
            spawn_rate: 5.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Dog).unwrap_or(&0),
        });
        self.biome_animals.insert(BiomeType::Catacombs, catacombs_animals);
        
        // Groves biome animals
        let mut groves_animals = Vec::new();
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Snake,
            spawn_rate: 6.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Snake).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Cobra,
            spawn_rate: 3.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Cobra).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Kingsnake,
            spawn_rate: 3.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Kingsnake).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::BlackMamba,
            spawn_rate: 2.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::BlackMamba).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Rat,
            spawn_rate: 15.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Rat).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Honeybadger,
            spawn_rate: 3.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Honeybadger).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::GrizzlyBear,
            spawn_rate: 1.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::GrizzlyBear).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::BlackBear,
            spawn_rate: 1.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::BlackBear).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Pig,
            spawn_rate: 2.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Pig).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Boar,
            spawn_rate: 1.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Boar).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Capybara,
            spawn_rate: 2.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Capybara).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Beaver,
            spawn_rate: 5.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Beaver).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::WaterBuffalo,
            spawn_rate: 2.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::WaterBuffalo).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::Yak,
            spawn_rate: 1.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::Yak).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::MallardDuck,
            spawn_rate: 4.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::MallardDuck).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::SheepRam,
            spawn_rate: 1.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::SheepRam).unwrap_or(&0),
        });
        groves_animals.push(AnimalSpawnData {
            animal_type: AnimalType::SheepEwe,
            spawn_rate: 1.0,
            sprite_index: *self.animal_sprites.get(&AnimalType::SheepEwe).unwrap_or(&0),
        });
        self.biome_animals.insert(BiomeType::Groves, groves_animals);
    }
    
    // Get a random animal for a specific biome based on spawn rates
    pub fn get_random_animal(&self, biome: BiomeType, rng: &mut impl Rng) -> Option<&AnimalSpawnData> {
        let biome_animals = self.biome_animals.get(&biome)?;
        
        if biome_animals.is_empty() {
            return None;
        }
        
        // Calculate total spawn rate for normalization
        let total_spawn_rate: f32 = biome_animals.iter().map(|a| a.spawn_rate).sum();
        
        // Generate a random value between 0 and the total spawn rate
        let random_value = rng.gen_range(0.0..total_spawn_rate);
        
        // Find the animal based on the random value and spawn rates
        let mut cumulative_rate = 0.0;
        for animal in biome_animals {
            cumulative_rate += animal.spawn_rate;
            if random_value <= cumulative_rate {
                return Some(animal);
            }
        }
        
        // Fallback to a random animal if something went wrong
        let index = rng.gen_range(0..biome_animals.len());
        Some(&biome_animals[index])
    }
}

// Function to spawn animals on the map
pub fn spawn_animals(
    commands: &mut Commands,
    map: &TileMap,
    texture_atlases: &crate::assets::TextureAtlases,
    animal_manager: &AnimalManager,
) {
    let mut rng = rand::thread_rng();
    
    // Get the biome for this map
    let biome = map.get_biome_at(0, 0); // All maps currently use a single biome
    
    // Find valid floor tiles for animal spawning
    let mut valid_positions = Vec::new();
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            if map.tiles[y][x] == TileType::Floor {
                // Check if this is a player spawn position or stairs
                let is_player_pos = map.get_spawn_position().0 == x && map.get_spawn_position().1 == y;
                let is_stairs = map.down_stairs_pos.map_or(false, |pos| pos.0 == x && pos.1 == y) ||
                               map.up_stairs_pos.map_or(false, |pos| pos.0 == x && pos.1 == y);
                
                if !is_player_pos && !is_stairs {
                    valid_positions.push((x as i32, y as i32));
                }
            }
        }
    }
    
    // Shuffle the valid positions
    valid_positions.shuffle(&mut rng);
    
    // Determine how many animals to spawn (up to MAX_ANIMALS_PER_MAP)
    let num_animals = rng.gen_range(0..=MAX_ANIMALS_PER_MAP);
    
    // Spawn the animals
    for _ in 0..num_animals {
        if valid_positions.is_empty() {
            break;
        }
        
        // Get a random position
        let pos = valid_positions.pop().unwrap();
        
        // Get a random animal for this biome
        if let Some(animal_data) = animal_manager.get_random_animal(biome, &mut rng) {
            let transform = Transform::from_xyz(
                pos.0 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                pos.1 as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                7.0  // Increased z-index to ensure animals render on top of all terrain and NPCs
            ).with_scale(Vec3::splat(1.0));
            
            // Get animal name
            let animal_name = animal_data.animal_type.get_name();
            
            // Spawn the animal entity as an NPC
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: texture_atlases.animals.clone(),
                    sprite: TextureAtlasSprite {
                        index: animal_data.sprite_index,
                        ..default()
                    },
                    transform,
                    ..default()
                },
                // Add both Animal and Npc components
                Animal {
                    animal_type: animal_data.animal_type,
                    hover: false,
                },
                // Add Npc component with animal-specific settings
                Npc {
                    name: format!("{} ({})", animal_name, animal_data.animal_type.get_name()),
                    dialog: vec![format!("A {} watches you cautiously.", animal_name)],
                    speaking: false,
                    dialog_text: format!("A {} watches you cautiously.", animal_name),
                    current_dialog_index: 0,
                    character_type: CharacterType::Generic,
                    animation_timer: Timer::from_seconds(0.3, TimerMode::Once),
                    original_scale: Vec3::splat(1.0),
                    wiggle_direction: 1.0,
                    wiggle_amount: 0.1,
                    is_animal: true,
                    animal_type: Some(animal_data.animal_type),
                },
                // Add marker component
                AnimalNpc,
                Position::new(pos.0, pos.1),
                AnimalAnimation {
                    start_pos: transform.translation,
                    target_pos: transform.translation,
                    ..default()
                },
            ));
            
            println!("Spawned {:?} at position: ({}, {})", animal_data.animal_type, pos.0, pos.1);
        }
    }
}

// System to handle mouse hover over animals
pub fn handle_animal_hover(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut animal_query: Query<(Entity, &mut Animal, &Transform, &Position)>,
    tooltip_query: Query<Entity, With<AnimalTooltip>>,
    asset_server: Res<AssetServer>,
) {
    // Get the window and camera
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    
    // Get the cursor position in the window
    if let Some(cursor_position) = window.cursor_position() {
        // Convert cursor position to world coordinates
        if let Some(world_position) = camera.viewport_to_world(camera_transform, cursor_position) {
            let world_pos = world_position.origin.truncate();
            
            // Check if the cursor is over any animal
            let mut hovered_animal = None;
            
            for (entity, mut animal, transform, position) in animal_query.iter_mut() {
                // Calculate the bounds of the animal sprite
                let animal_pos = Vec2::new(
                    position.x as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                    position.y as f32 * TILE_SIZE + (TILE_SIZE / 2.0)
                );
                
                let half_size = TILE_SIZE / 2.0;
                let min_x = animal_pos.x - half_size;
                let max_x = animal_pos.x + half_size;
                let min_y = animal_pos.y - half_size;
                let max_y = animal_pos.y + half_size;
                
                // Check if the cursor is within the bounds
                if world_pos.x >= min_x && world_pos.x <= max_x && 
                   world_pos.y >= min_y && world_pos.y <= max_y {
                    // Set hover state to true
                    animal.hover = true;
                    hovered_animal = Some((entity, animal.animal_type, transform.translation));
                } else {
                    // Set hover state to false
                    animal.hover = false;
                }
            }
            
            // Remove any existing tooltips
            for entity in tooltip_query.iter() {
                commands.entity(entity).despawn();
            }
            
            // Create a tooltip for the hovered animal
            if let Some((_, animal_type, position)) = hovered_animal {
                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(
                            animal_type.get_name(),
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Light.ttf"),
                                font_size: 14.0,
                                color: Color::WHITE,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                        transform: Transform::from_translation(
                            position + Vec3::new(0.0, TILE_SIZE, 15.0) // Position above the animal
                        ),
                        ..default()
                    },
                    AnimalTooltip,
                ));
            }
        }
    }
}

// System to handle animal movement based on turns
pub fn move_animals_system(
    mut commands: Commands,
    mut param_set: ParamSet<(
        Query<(Entity, &Animal, &Npc, &Position, &mut Transform, &mut AnimalAnimation, &mut TextureAtlasSprite), With<AnimalNpc>>,
        Query<&Position, With<crate::components::Player>>
    )>,
    map: Res<TileMap>,
    game_turn: Res<GameTurn>,
    mut local: Local<u32>, // Add a local resource to track the last turn animals moved
) {
    // Only move animals if this is a new turn
    if game_turn.current_turn == 0 || game_turn.current_turn == *local {
        return;
    }
    
    // Store the current turn so we don't process it again
    *local = game_turn.current_turn;
    
    // Get player position
    let player_pos = if let Ok(pos) = param_set.p1().get_single() {
        *pos // Now works because Position implements Copy
    } else {
        return; // No player found
    };
    
    // Process animal movements
    let mut animal_query = param_set.p0();
    for (entity, animal, _npc, position, _transform, mut animation, mut sprite) in animal_query.iter_mut() {
        // Different movement behavior based on animal type
        let target_pos = match animal.animal_type {
            // For predator-type animals
            AnimalType::GrizzlyBear | AnimalType::BlackBear | AnimalType::Dog | AnimalType::Honeybadger => {
                // Predators move toward the player if within range
                let dx = player_pos.x - position.x;
                let dy = player_pos.y - position.y;
                
                // Only chase if within 10 tiles
                if dx.abs() + dy.abs() <= 10 {
                    let mut target_pos = *position; // Now works because Position implements Copy
                    
                    // Move one step in either x or y direction toward player
                    if dx.abs() > dy.abs() {
                        // Move horizontally
                        target_pos.x += if dx > 0 { 1 } else { -1 };
                    } else {
                        // Move vertically
                        target_pos.y += if dy > 0 { 1 } else { -1 };
                    }
                    
                    target_pos
                } else {
                    // Random movement if player is too far
                    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
                    let dir = directions[rand::random::<usize>() % 4];
                    Position {
                        x: position.x + dir.0,
                        y: position.y + dir.1,
                    }
                }
            },
            // Other animals move randomly
            _ => {
                let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
                let dir = directions[rand::random::<usize>() % 4];
                Position {
                    x: position.x + dir.0,
                    y: position.y + dir.1,
                }
            }
        };
        
        // Check if the target position is valid (walkable)
        if map.is_position_walkable(target_pos.x, target_pos.y) {
            println!("Animal moving from ({}, {}) to ({}, {}) on turn {}", 
                     position.x, position.y, target_pos.x, target_pos.y, game_turn.current_turn);
            
            // Determine horizontal movement direction for sprite flipping
            let moving_right = target_pos.x > position.x;
            let moving_left = target_pos.x < position.x;
            
            // Only update facing direction for horizontal movement
            if moving_right || moving_left {
                // Set the facing direction in the animation component
                // Since sprites initially face left, facing_right should be true when moving right
                animation.facing_right = moving_right;
                
                // Animal sprites initially face left, so:
                // - When moving right, we need to flip the sprite (flip_x = true)
                // - When moving left, we don't flip the sprite (flip_x = false)
                sprite.flip_x = moving_right;
                
                println!("Flipping animal sprite to face {}", if moving_right { "right" } else { "left" });
            }
            
            // Start the animation
            animation.is_moving = true;
            animation.start_pos = Vec3::new(
                position.x as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                position.y as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                7.0  // Increased z-index to ensure animals render on top of all terrain and NPCs
            );
            animation.target_pos = Vec3::new(
                target_pos.x as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                target_pos.y as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                7.0  // Increased z-index to ensure animals render on top of all terrain and NPCs
            );
            animation.animation_timer.reset();
            
            // Update the position component
            commands.entity(entity).insert(target_pos);
        }
    }
}

// System to animate animal movement
pub fn animate_animal_movement(
    time: Res<Time>,
    mut animal_query: Query<(&Position, &mut Transform, &mut AnimalAnimation, &mut TextureAtlasSprite), With<Animal>>,
    _animation_state: ResMut<crate::AnimationState>,
) {
    // Track if any animal is currently moving (for debugging purposes)
    let mut _any_animal_moving = false;
    
    for (_position, mut transform, mut animation, mut sprite) in animal_query.iter_mut() {
        // If currently animating, continue the animation
        if animation.is_moving {
            // Track that at least one animal is moving
            _any_animal_moving = true;
            
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
            
            // Ensure sprite is flipped correctly based on facing direction
            // Since sprites initially face left, we flip when facing right
            sprite.flip_x = animation.facing_right;
            
            // Check if the animation is complete
            if animation.animation_timer.finished() {
                // Reset the animation state
                animation.is_moving = false;
                transform.rotation = Quat::IDENTITY; // Reset rotation
                
                // Set the final position exactly
                transform.translation = animation.target_pos;
                
                println!("Animal animation completed");
            }
        }
    }
    
    // Note: We don't set animation_state.animation_in_progress here
    // This allows animal animations to run independently of player movement
} 