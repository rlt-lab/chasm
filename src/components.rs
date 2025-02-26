use bevy::prelude::*;
use crate::biome::{BiomeType, TileWalkability};
use crate::map::TileType;

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
}

impl Default for Npc {
    fn default() -> Self {
        Self {
            speaking: false,
            dialog_text: "Hello!".to_string(),
            name: "NPC".to_string(),
            dialog: vec!["Hello!".to_string()],
            current_dialog_index: 0,
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
