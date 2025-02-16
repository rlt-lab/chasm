use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Tile {
    pub tile_type: TileType,
}

#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Floor,
    Wall,
    Door,
}

