use bevy::prelude::*;
use rand::Rng;

pub const MAP_WIDTH: usize = 45;
pub const MAP_HEIGHT: usize = 25;

#[derive(Component, Resource, Clone)]
pub struct TileMap {
    pub tiles: [[TileType; MAP_WIDTH]; MAP_HEIGHT],
    pub spawn_position: (usize, usize),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TileType {
    Floor,
    Wall,
}

#[derive(Debug, Clone)]
struct Room {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Room {
    fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Room { x, y, width, height }
    }

    fn overlaps(&self, other: &Room) -> bool {
        let self_x2 = self.x + self.width;
        let self_y2 = self.y + self.height;
        let other_x2 = other.x + other.width;
        let other_y2 = other.y + other.height;

        !(self_x2 < other.x || self.x > other_x2 || 
        self_y2 < other.y || self.y > other_y2)
    }

    fn center(&self) -> (usize, usize) {
        (self.x + self.width / 2, 
        self.y + self.height / 2)
    }

    fn carve_irregular_room(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rng: &mut impl Rng) {
        // Carve out the basic room shape first
        for y in self.y..self.y + self.height {
            for x in self.x..self.x + self.width {
                // 90% chance of being floor for a more solid but slightly irregular room
                if rng.gen_bool(0.9) {
                    tiles[y][x] = TileType::Floor;
                }
            }
        }

        // Ensure the center and critical paths are always floor
        let (center_x, center_y) = self.center();
        for y in self.y + 1..self.y + self.height - 1 {
            tiles[y][center_x] = TileType::Floor;
        }
        for x in self.x + 1..self.x + self.width - 1 {
            tiles[center_y][x] = TileType::Floor;
        }
    }

    fn add_internal_features(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rng: &mut impl Rng) {
        // Only add features to larger rooms
        if self.width < 8 || self.height < 8 {
            return;
        }

        match rng.gen_range(0..4) {
            0 => { // Add pillars in corners
                let pillar_positions = [
                    (self.x + 2, self.y + 2),
                    (self.x + self.width - 3, self.y + 2),
                    (self.x + 2, self.y + self.height - 3),
                    (self.x + self.width - 3, self.y + self.height - 3),
                ];
                for (px, py) in pillar_positions.iter() {
                    tiles[*py][*px] = TileType::Wall;
                }
            }
            1 => { // Add a central pillar
                let (cx, cy) = self.center();
                tiles[cy][cx] = TileType::Wall;
            }
            2 => { // Add random internal walls
                for _ in 0..3 {
                    let x = rng.gen_range(self.x + 2..self.x + self.width - 2);
                    let y = rng.gen_range(self.y + 2..self.y + self.height - 2);
                    tiles[y][x] = TileType::Wall;
                }
            }
            _ => {} // Leave room empty
        }
    }
}
impl TileMap {
    fn generate_first_room(rng: &mut impl Rng) -> Room {
        let width = rng.gen_range(15..25);  // Much larger possible width
        let height = rng.gen_range(12..20); // Much larger possible height
        
        let max_x = MAP_WIDTH - width - 1;
        let max_y = MAP_HEIGHT - height - 1;
        let x = rng.gen_range(1..=max_x);
        let y = rng.gen_range(1..=max_y);
        Room::new(x, y, width, height)
    }

    fn generate_room(rng: &mut impl Rng) -> Room {
        let room_type = rng.gen_range(0..4);
        
        match room_type {
            0 => { // Massive room
                let width = rng.gen_range(20..30);
                let height = rng.gen_range(15..20);
                let max_x = MAP_WIDTH - width - 1;
                let max_y = MAP_HEIGHT - height - 1;
                let x = rng.gen_range(1..=max_x);
                let y = rng.gen_range(1..=max_y);
                Room::new(x, y, width, height)
            }
            1 => { // Large irregular room
                let width = rng.gen_range(15..20);
                let height = rng.gen_range(12..16);
                let max_x = MAP_WIDTH - width - 1;
                let max_y = MAP_HEIGHT - height - 1;
                let x = rng.gen_range(1..=max_x);
                let y = rng.gen_range(1..=max_y);
                Room::new(x, y, width, height)
            }
            2 => { // Extra wide room
                let width = rng.gen_range(18..25);
                let height = rng.gen_range(8..12);
                let max_x = MAP_WIDTH - width - 1;
                let max_y = MAP_HEIGHT - height - 1;
                let x = rng.gen_range(1..=max_x);
                let y = rng.gen_range(1..=max_y);
                Room::new(x, y, width, height)
            }
            _ => { // Tall room
                let width = rng.gen_range(8..12);
                let height = rng.gen_range(15..20);
                let max_x = MAP_WIDTH - width - 1;
                let max_y = MAP_HEIGHT - height - 1;
                let x = rng.gen_range(1..=max_x);
                let y = rng.gen_range(1..=max_y);
                Room::new(x, y, width, height)
            }
        }
    }


    fn connect_rooms(tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], 
                room1: &Room, room2: &Room) {
        let (x1, y1) = room1.center();
        let (x2, y2) = room2.center();

        // Draw horizontal corridor
        let xstart = x1.min(x2);
        let xend = x1.max(x2);
        for x in xstart..=xend {
            tiles[y1][x] = TileType::Floor;
        }

        // Draw vertical corridor
        let ystart = y1.min(y2);
        let yend = y1.max(y2);
        for y in ystart..=yend {
            tiles[y][x2] = TileType::Floor;
        }
    }

    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut tiles = [[TileType::Wall; MAP_WIDTH]; MAP_HEIGHT];

        let num_rooms = rng.gen_range(3..=6); // Reduced from 5..=8 to allow for larger rooms
        let mut rooms = Vec::new();
        let mut attempts = 0;
        const MAX_ATTEMPTS: i32 = 1000;

        // Keep trying to add rooms until we have enough
        while rooms.len() < num_rooms && attempts < MAX_ATTEMPTS {
            attempts += 1;
            let new_room = if rooms.is_empty() {
                Self::generate_first_room(&mut rng)
            } else {
                Self::generate_room(&mut rng)
            };
            
            // Verify room bounds
            if new_room.x + new_room.width >= MAP_WIDTH || 
            new_room.y + new_room.height >= MAP_HEIGHT {
                continue;
            }
            
            // Check if room overlaps with any existing rooms
            if !rooms.iter().any(|room: &Room| room.overlaps(&new_room)) {
                // Carve out the room
                // First set the area to walls
                for y in new_room.y..new_room.y + new_room.height {
                    for x in new_room.x..new_room.x + new_room.width {
                        tiles[y][x] = TileType::Wall;
                    }
                }

                // Then carve out the irregular room shape
                new_room.carve_irregular_room(&mut tiles, &mut rng);
                
                new_room.add_internal_features(&mut tiles, &mut rng);
                
                // Connect to multiple previous rooms
                if !rooms.is_empty() {
                    // Always connect to the last room
                    Self::connect_rooms(&mut tiles, rooms.last().unwrap(), &new_room);
                    
                    // Randomly connect to other rooms
                    for i in 0..rooms.len()-1 {
                        if rng.gen_bool(0.35) { // Increase connection chance to 35%
                            Self::connect_rooms(&mut tiles, &rooms[i], &new_room);
                        }
                    }
                }
                
                rooms.push(new_room);
            }
        }

        // Ensure we have at least one room
        if rooms.is_empty() {
            // Create a fallback room in a safe location
            let fallback_room = Room::new(MAP_WIDTH/4, MAP_HEIGHT/4, 15, 15); // Make fallback room larger
            for y in fallback_room.y..fallback_room.y+fallback_room.height {
                for x in fallback_room.x..fallback_room.x+fallback_room.width {
                    tiles[y][x] = TileType::Floor;
                }
            }
            rooms.push(fallback_room);
        }

        // Create walls around the edges
        for y in 0..MAP_HEIGHT {
            tiles[y][0] = TileType::Wall;
            tiles[y][MAP_WIDTH-1] = TileType::Wall;
        }
        for x in 0..MAP_WIDTH {
            tiles[0][x] = TileType::Wall;
            tiles[MAP_HEIGHT-1][x] = TileType::Wall;
        }

        let spawn_position = rooms[0].center();
        TileMap { tiles, spawn_position }
    }
pub fn get_spawn_position(&self) -> (usize, usize) {
    self.spawn_position
}
}

pub fn spawn_map(commands: &mut Commands) -> Entity {
    commands.spawn(TileMap::new()).id()
}

