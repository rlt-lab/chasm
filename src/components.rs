use bevy::prelude::*;
use crate::biome::{BiomeType, TileWalkability};
use crate::map::TileType;
use crate::dialogue::CharacterType;

#[derive(Component, Debug, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component)]
pub struct PlayerAnimation {
    pub is_moving: bool,
    pub start_pos: Vec3,
    pub target_pos: Vec3,
    pub animation_timer: Timer,
    pub hop_height: f32,
    pub wobble_amount: f32,
    pub wobble_direction: f32,
    pub rapid_press_count: u8,
    pub continuous_movement_timer: Timer,
    pub last_movement_direction: Option<MovementDirection>,
    pub queued_direction: Option<MovementDirection>,
}

impl Default for PlayerAnimation {
    fn default() -> Self {
        Self {
            is_moving: false,
            start_pos: Vec3::ZERO,
            target_pos: Vec3::ZERO,
            animation_timer: Timer::from_seconds(0.2, TimerMode::Once),
            hop_height: 10.0,
            wobble_amount: 0.3,
            wobble_direction: 1.0,
            rapid_press_count: 0,
            continuous_movement_timer: Timer::from_seconds(0.5, TimerMode::Once),
            last_movement_direction: None,
            queued_direction: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Debug)]
pub struct Tile {
    pub tile_type: TileType,
    pub walkability: TileWalkability,
    pub biome: BiomeType,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            tile_type: TileType::Floor,
            walkability: TileWalkability::Walkable,
            biome: BiomeType::Caves,
        }
    }
}

#[derive(Component, Debug)]
pub struct Npc {
    pub speaking: bool,
    pub dialog_text: String,
    pub name: String,
    pub dialog: Vec<String>,
    pub current_dialog_index: usize,
    pub character_type: CharacterType,
    pub animation_timer: Timer,
    pub original_scale: Vec3,
    pub wiggle_direction: f32,
    pub wiggle_amount: f32,
}

impl Default for Npc {
    fn default() -> Self {
        Self {
            speaking: false,
            dialog_text: "Hello!".to_string(),
            name: "NPC".to_string(),
            dialog: vec!["Hello!".to_string()],
            current_dialog_index: 0,
            character_type: CharacterType::Generic,
            animation_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            original_scale: Vec3::splat(1.0),
            wiggle_direction: 1.0,
            wiggle_amount: 0.05,
        }
    }
}

#[derive(Component, Debug)]
pub struct DialogBox {
    pub text: String,
    pub visible: bool,
}

impl Default for DialogBox {
    fn default() -> Self {
        Self {
            text: String::new(),
            visible: false,
        }
    }
}
