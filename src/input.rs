use bevy::prelude::*;
use crate::map::{TileMap, TileType, MAP_SIZE};

#[derive(Resource, Default)]
pub struct InputState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    interact: bool,
    attack: bool,
}

#[derive(Component, Default)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
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
}

pub const TILE_SIZE: f32 = 32.0;

pub fn move_player(
    mut query: Query<&mut GridPosition, With<Player>>,
    input: Res<InputState>,
    tilemap: Res<TileMap>,
) {
    for mut grid_pos in &mut query {
        let mut new_pos = GridPosition::new(grid_pos.x, grid_pos.y);
        
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
        if new_pos.x >= 0 && new_pos.x < MAP_SIZE as i32 &&
        new_pos.y >= 0 && new_pos.y < MAP_SIZE as i32 {
            // Check if the new position would collide with a wall
            if tilemap.tiles[new_pos.y as usize][new_pos.x as usize] != TileType::Wall {
                // Apply the movement only if valid
                grid_pos.x = new_pos.x;
                grid_pos.y = new_pos.y;
            }
        }
    }
}

#[derive(Component)]
pub struct Player;

