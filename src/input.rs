use bevy::prelude::*;
use crate::map::{TileMap, TileType, MAP_WIDTH, MAP_HEIGHT};
use crate::components::{Position, Player, Tile, MovementDirection, PlayerAnimation};
use crate::biome::TileWalkability;
use crate::AnimationState;

#[derive(Resource, Default)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub interact: bool,
    pub attack: bool,
    pub regenerate_map: bool,
    pub last_key_press_time: f64,
    pub last_direction: Option<MovementDirection>,
    pub continuous_movement: bool,
    pub use_stairs_down: bool,
    pub use_stairs_up: bool,
}

pub fn handle_input(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut input_state: ResMut<InputState>,
    animation_state: Res<AnimationState>,
) {
    // Reset movement flags
    input_state.up = false;
    input_state.down = false;
    input_state.left = false;
    input_state.right = false;
    input_state.interact = false;
    input_state.attack = false;
    input_state.regenerate_map = false;
    
    // Check for movement keys - only set flags if no animation is in progress
    // or if we're handling continuous movement
    let can_process_movement = !animation_state.animation_in_progress || input_state.continuous_movement;
    
    if can_process_movement {
        if keyboard.just_pressed(KeyCode::W) || keyboard.just_pressed(KeyCode::Up) {
            input_state.up = true;
            input_state.last_key_press_time = time.elapsed_seconds_f64();
            input_state.last_direction = Some(MovementDirection::Up);
        }
        if keyboard.just_pressed(KeyCode::S) || keyboard.just_pressed(KeyCode::Down) {
            input_state.down = true;
            input_state.last_key_press_time = time.elapsed_seconds_f64();
            input_state.last_direction = Some(MovementDirection::Down);
        }
        if keyboard.just_pressed(KeyCode::A) || keyboard.just_pressed(KeyCode::Left) {
            input_state.left = true;
            input_state.last_key_press_time = time.elapsed_seconds_f64();
            input_state.last_direction = Some(MovementDirection::Left);
        }
        if keyboard.just_pressed(KeyCode::D) || keyboard.just_pressed(KeyCode::Right) {
            input_state.right = true;
            input_state.last_key_press_time = time.elapsed_seconds_f64();
            input_state.last_direction = Some(MovementDirection::Right);
        }
    }
    
    // Always track the last direction for continuous movement, even if we can't process movement yet
    if keyboard.just_pressed(KeyCode::W) || keyboard.just_pressed(KeyCode::Up) {
        input_state.last_direction = Some(MovementDirection::Up);
        input_state.last_key_press_time = time.elapsed_seconds_f64();
    }
    if keyboard.just_pressed(KeyCode::S) || keyboard.just_pressed(KeyCode::Down) {
        input_state.last_direction = Some(MovementDirection::Down);
        input_state.last_key_press_time = time.elapsed_seconds_f64();
    }
    if keyboard.just_pressed(KeyCode::A) || keyboard.just_pressed(KeyCode::Left) {
        input_state.last_direction = Some(MovementDirection::Left);
        input_state.last_key_press_time = time.elapsed_seconds_f64();
    }
    if keyboard.just_pressed(KeyCode::D) || keyboard.just_pressed(KeyCode::Right) {
        input_state.last_direction = Some(MovementDirection::Right);
        input_state.last_key_press_time = time.elapsed_seconds_f64();
    }
    
    // Check for continuous movement (holding keys)
    input_state.continuous_movement = false;
    if keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up) {
        input_state.continuous_movement = true;
        if input_state.last_direction.is_none() {
            input_state.last_direction = Some(MovementDirection::Up);
        }
    }
    if keyboard.pressed(KeyCode::S) || keyboard.pressed(KeyCode::Down) {
        input_state.continuous_movement = true;
        if input_state.last_direction.is_none() {
            input_state.last_direction = Some(MovementDirection::Down);
        }
    }
    if keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left) {
        input_state.continuous_movement = true;
        if input_state.last_direction.is_none() {
            input_state.last_direction = Some(MovementDirection::Left);
        }
    }
    if keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right) {
        input_state.continuous_movement = true;
        if input_state.last_direction.is_none() {
            input_state.last_direction = Some(MovementDirection::Right);
        }
    }
    
    // Check for map regeneration (SHIFT+R)
    if keyboard.pressed(KeyCode::ShiftLeft) && keyboard.just_pressed(KeyCode::R) {
        input_state.regenerate_map = true;
    }
    
    // Check for stair navigation
    input_state.use_stairs_down = keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::S);
    input_state.use_stairs_up = keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::W);
}

pub const TILE_SIZE: f32 = 32.0;

pub fn move_player(
    mut query: Query<&mut Position, With<Player>>,
    input: Res<InputState>,
    tilemap: Res<TileMap>,
    tile_query: Query<(&crate::map::TilePos, &Tile), Without<Player>>,
    animation_state: Res<AnimationState>,
) {
    // Skip movement if an animation is in progress
    if animation_state.animation_in_progress {
        return;
    }

    for mut pos in &mut query {
        let mut new_pos = Position::new(pos.x, pos.y);
        
        if input.up {
            new_pos.y += 1;
        } else if input.down {
            new_pos.y -= 1;
        } else if input.left {
            new_pos.x -= 1;
        } else if input.right {
            new_pos.x += 1;
        }

        // Check if the new position is within bounds
        if new_pos.x >= 0 && new_pos.x < MAP_WIDTH as i32 &&
        new_pos.y >= 0 && new_pos.y < MAP_HEIGHT as i32 {
            // Default to not allowing movement unless we find a tile entity that says otherwise
            let mut can_move = false;
            
            // Check for walkability information from tile entities
            let mut found_tile = false;
            for (tile_pos, tile) in tile_query.iter() {
                if tile_pos.x == new_pos.x && tile_pos.y == new_pos.y {
                    found_tile = true;
                    // Use the tile's walkability property
                    can_move = match tile.walkability {
                        TileWalkability::Walkable => true,
                        TileWalkability::Blocked => false,
                        TileWalkability::Door => {
                            // Doors can be walked through if the player presses the interact key
                            if input.interact {
                                true
                            } else {
                                false
                            }
                        }
                    };
                    break;
                }
            }
            
            // If no tile entity was found, fall back to the tilemap data
            // This should rarely happen if tiles are spawned correctly
            if !found_tile {
                let tile_type = tilemap.tiles[new_pos.y as usize][new_pos.x as usize];
                can_move = match tile_type {
                    TileType::Floor => true,
                    TileType::Wall => false,
                    TileType::Door => input.interact, // Only if interact is pressed
                    TileType::SecretDoor => input.interact, // Only if interact is pressed
                    TileType::StairsDown => true,
                    TileType::StairsUp => true,
                };
                println!("Warning: No tile entity found at ({}, {}), using tilemap data", new_pos.x, new_pos.y);
            }
            
            // Apply the movement only if valid
            if can_move {
                pos.x = new_pos.x;
                pos.y = new_pos.y;
            }
        }
    }
}

// Add a new system to queue up the next movement direction
pub fn queue_next_movement(
    keyboard: Res<Input<KeyCode>>,
    animation_state: Res<AnimationState>,
    mut player_query: Query<&mut PlayerAnimation, With<Player>>,
) {
    // Only queue movements if an animation is in progress
    if !animation_state.animation_in_progress {
        return;
    }
    
    // Check for a player animation component
    if let Ok(mut animation) = player_query.get_single_mut() {
        // Check for movement keys and queue the direction
        if keyboard.just_pressed(KeyCode::W) || keyboard.just_pressed(KeyCode::Up) {
            animation.queued_direction = Some(MovementDirection::Up);
            println!("Queued UP movement");
        } else if keyboard.just_pressed(KeyCode::S) || keyboard.just_pressed(KeyCode::Down) {
            animation.queued_direction = Some(MovementDirection::Down);
            println!("Queued DOWN movement");
        } else if keyboard.just_pressed(KeyCode::A) || keyboard.just_pressed(KeyCode::Left) {
            animation.queued_direction = Some(MovementDirection::Left);
            println!("Queued LEFT movement");
        } else if keyboard.just_pressed(KeyCode::D) || keyboard.just_pressed(KeyCode::Right) {
            animation.queued_direction = Some(MovementDirection::Right);
            println!("Queued RIGHT movement");
        }
    }
}

