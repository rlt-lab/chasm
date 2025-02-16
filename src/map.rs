use bevy::prelude::*;

pub const MAP_WIDTH: usize = 45;
pub const MAP_HEIGHT: usize = 25;
#[derive(Component, Resource, Clone)]
pub struct TileMap {
    pub tiles: [[TileType; MAP_WIDTH]; MAP_HEIGHT],
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TileType {
    Floor,
    Wall,
}

impl TileMap {
    pub fn new() -> Self {
        let mut tiles = [[TileType::Floor; MAP_WIDTH]; MAP_HEIGHT];
        
        // Create walls around the edges
        for y in 0..MAP_HEIGHT {
            tiles[y][0] = TileType::Wall;
            tiles[y][MAP_WIDTH-1] = TileType::Wall;
        }
        for x in 0..MAP_WIDTH {
            tiles[0][x] = TileType::Wall;
            tiles[MAP_HEIGHT-1][x] = TileType::Wall;
        }
        
        TileMap { tiles }
    }
}

pub fn spawn_map(commands: &mut Commands) -> Entity {
    commands.spawn(TileMap::new()).id()
}

