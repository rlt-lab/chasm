use bevy::prelude::*;
use crate::map::{TileMap, TileType, MAP_WIDTH, MAP_HEIGHT};
use crate::components::{Position, Player, Tile};
use crate::biome::TileWalkability;

#[derive(Resource, Default)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub interact: bool,
    pub attack: bool,
    pub regenerate_map: bool,
    pub move_direction: Option<Direction>,
    pub use_stairs_down: bool,
    pub use_stairs_up: bool,
}

pub fn handle_input(
    keyboard: Res<Input<KeyCode>>,
    mut input_state: ResMut<InputState>,
) {
    input_state.up = keyboard.just_pressed(KeyCode::Up);
    input_state.down = keyboard.just_pressed(KeyCode::Down);
    input_state.left = keyboard.just_pressed(KeyCode::Left);
    input_state.right = keyboard.just_pressed(KeyCode::Right);
    input_state.interact = keyboard.just_pressed(KeyCode::E);
    input_state.attack = keyboard.just_pressed(KeyCode::F);
    input_state.regenerate_map = keyboard.pressed(KeyCode::ShiftLeft) && keyboard.just_pressed(KeyCode::R);
    
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
) {
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

