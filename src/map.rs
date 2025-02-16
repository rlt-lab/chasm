use bevy::prelude::*;

pub const MAP_SIZE: usize = 20;

#[derive(Component, Resource, Clone)]
pub struct TileMap {
    pub tiles: [[TileType; MAP_SIZE]; MAP_SIZE],
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TileType {
    Floor,
    Wall,
}

impl TileMap {
    pub fn new() -> Self {
        let mut tiles = [[TileType::Floor; MAP_SIZE]; MAP_SIZE];
        
        // Create walls around the edges
        for x in 0..MAP_SIZE {
            tiles[x][0] = TileType::Wall;
            tiles[x][MAP_SIZE-1] = TileType::Wall;
            tiles[0][x] = TileType::Wall;
            tiles[MAP_SIZE-1][x] = TileType::Wall;
        }
        
        TileMap { tiles }
    }
}

pub fn spawn_map(commands: &mut Commands) -> Entity {
    commands.spawn(TileMap::new()).id()
}

