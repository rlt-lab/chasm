use bevy::prelude::*;
use crate::map::{TileMap, TileType, MAP_WIDTH, MAP_HEIGHT};
use crate::components::{Position, Player};
#[derive(Resource, Default)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub interact: bool,
    pub attack: bool,
    pub regenerate_map: bool,
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
}

pub const TILE_SIZE: f32 = 32.0;

pub fn move_player(
    mut query: Query<&mut Position, With<Player>>,
    input: Res<InputState>,
    tilemap: Res<TileMap>,
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
            // Check if the new position would collide with a wall
            if tilemap.tiles[new_pos.y as usize][new_pos.x as usize] != TileType::Wall {
                // Apply the movement only if valid
                pos.x = new_pos.x;
                pos.y = new_pos.y;
            }
        }
    }
}

