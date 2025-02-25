use bevy::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;
use crate::biome::BiomeType;

pub const MAP_WIDTH: usize = 45;
pub const MAP_HEIGHT: usize = 25;

#[derive(Component, Resource, Clone)]
pub struct TileMap {
    pub tiles: [[TileType; MAP_WIDTH]; MAP_HEIGHT],
    pub biomes: [[BiomeType; MAP_WIDTH]; MAP_HEIGHT],
    pub spawn_position: (usize, usize),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TileType {
    Floor,
    Wall,
    Door,
    SecretDoor,
}

// Represents a rectangular room or section of the map
#[derive(Debug, Clone)]
struct Room {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    room_type: RoomType,
}

#[derive(Debug, Clone, PartialEq)]
enum RoomType {
    Rectangular,
    Circular,
    CrossShaped,
    LShaped,
    Pillared,
    SmallChamber,
    LargeHall,
}

#[derive(Debug, Clone, PartialEq)]
enum RoomSize {
    Small,
    Medium,
    Large,
}

impl Room {
    fn new(x: usize, y: usize, width: usize, height: usize, room_type: RoomType) -> Self {
        Room { x, y, width, height, room_type }
    }

    fn size(&self) -> RoomSize {
        let area = self.width * self.height;
        if area < 36 {  // Less than 6x6
            RoomSize::Small
        } else if area < 81 {  // Less than 9x9
            RoomSize::Medium
        } else {
            RoomSize::Large
        }
    }

    fn center(&self) -> (usize, usize) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    // Check if this room overlaps with another room
    fn overlaps(&self, other: &Room) -> bool {
        // Add a buffer of 1 tile to ensure rooms aren't directly adjacent
        let self_x2 = self.x + self.width + 1;
        let self_y2 = self.y + self.height + 1;
        let other_x2 = other.x + other.width + 1;
        let other_y2 = other.y + other.height + 1;

        !(self_x2 < other.x || self.x > other_x2 || 
        self_y2 < other.y || self.y > other_y2)
    }

    // Carve a room into the map based on its type
    fn carve(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rng: &mut impl Rng) {
        match self.room_type {
            RoomType::Rectangular => self.carve_rectangular(tiles),
            RoomType::Circular => self.carve_circular(tiles),
            RoomType::CrossShaped => self.carve_cross_shaped(tiles),
            RoomType::LShaped => self.carve_l_shaped(tiles),
            RoomType::Pillared => self.carve_pillared(tiles, rng),
            RoomType::SmallChamber => self.carve_small_chamber(tiles),
            RoomType::LargeHall => self.carve_large_hall(tiles, rng),
        }
    }

    // Carve a basic rectangular room
    fn carve_rectangular(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT]) {
        for y in self.y..self.y + self.height {
            for x in self.x..self.x + self.width {
                if y > 0 && y < MAP_HEIGHT - 1 && x > 0 && x < MAP_WIDTH - 1 {
                    tiles[y][x] = TileType::Floor;
                }
            }
        }
    }

    // Carve a circular room
    fn carve_circular(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT]) {
        let center_x = self.x + self.width / 2;
        let center_y = self.y + self.height / 2;
        let radius_x = self.width as f32 / 2.0;
        let radius_y = self.height as f32 / 2.0;

        for y in self.y..self.y + self.height {
            for x in self.x..self.x + self.width {
                if y > 0 && y < MAP_HEIGHT - 1 && x > 0 && x < MAP_WIDTH - 1 {
                    // Calculate normalized distance from center
                    let dx = (x as f32 - center_x as f32) / radius_x;
                    let dy = (y as f32 - center_y as f32) / radius_y;
                    let distance = dx * dx + dy * dy;

                    // If inside the ellipse, make it a floor
                    if distance <= 1.0 {
                        tiles[y][x] = TileType::Floor;
                    }
                }
            }
        }
    }

    // Carve a cross-shaped room
    fn carve_cross_shaped(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT]) {
        let third_width = self.width / 3;
        let third_height = self.height / 3;

        // Carve the horizontal bar of the cross
        for y in self.y + third_height..self.y + 2 * third_height {
            for x in self.x..self.x + self.width {
                if y > 0 && y < MAP_HEIGHT - 1 && x > 0 && x < MAP_WIDTH - 1 {
                    tiles[y][x] = TileType::Floor;
                }
            }
        }

        // Carve the vertical bar of the cross
        for y in self.y..self.y + self.height {
            for x in self.x + third_width..self.x + 2 * third_width {
                if y > 0 && y < MAP_HEIGHT - 1 && x > 0 && x < MAP_WIDTH - 1 {
                    tiles[y][x] = TileType::Floor;
                }
            }
        }
    }

    // Carve an L-shaped room
    fn carve_l_shaped(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT]) {
        let half_width = self.width / 2;
        let half_height = self.height / 2;

        // Carve the horizontal part of the L
        for y in self.y..self.y + half_height {
            for x in self.x..self.x + self.width {
                if y > 0 && y < MAP_HEIGHT - 1 && x > 0 && x < MAP_WIDTH - 1 {
                    tiles[y][x] = TileType::Floor;
                }
            }
        }

        // Carve the vertical part of the L
        for y in self.y + half_height..self.y + self.height {
            for x in self.x..self.x + half_width {
                if y > 0 && y < MAP_HEIGHT - 1 && x > 0 && x < MAP_WIDTH - 1 {
                    tiles[y][x] = TileType::Floor;
                }
            }
        }
    }

    // Carve a room with pillars
    fn carve_pillared(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rng: &mut impl Rng) {
        // First carve the basic rectangular room
        self.carve_rectangular(tiles);

        // Only add pillars if the room is large enough
        if self.width < 7 || self.height < 7 {
            return;
        }

        // Determine number of pillars based on room size
        let num_pillars = rng.gen_range(1..=4);
        
        for _ in 0..num_pillars {
            // Ensure pillars are not at the edges
            let pillar_x = rng.gen_range(self.x + 2..self.x + self.width - 2);
            let pillar_y = rng.gen_range(self.y + 2..self.y + self.height - 2);
            
            // Create a 2x2 pillar
            for py in pillar_y..pillar_y + 2 {
                for px in pillar_x..pillar_x + 2 {
                    if py < MAP_HEIGHT && px < MAP_WIDTH {
                        tiles[py][px] = TileType::Wall;
                    }
                }
            }
        }
    }

    // Carve a small chamber (simple, possibly irregular shape)
    fn carve_small_chamber(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT]) {
        // Basic rectangular shape for small chambers
        self.carve_rectangular(tiles);
        
        // Sometimes make one corner rounded
        if self.width >= 4 && self.height >= 4 {
            // Choose a corner to round (top-right in this case)
            let corner_x = self.x + self.width - 1;
            let corner_y = self.y;
            
            // Make the corner a wall again
            if corner_x < MAP_WIDTH && corner_y < MAP_HEIGHT {
                tiles[corner_y][corner_x] = TileType::Wall;
            }
        }
    }
    
    // Carve a large hall with possible features
    fn carve_large_hall(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rng: &mut impl Rng) {
        // First carve the basic rectangular room
        self.carve_rectangular(tiles);
        
        // Only add features if the room is large enough
        if self.width < 8 || self.height < 8 {
            return;
        }
        
        // Choose a feature type for the large hall
        match rng.gen_range(0..4) {
            0 => self.add_central_feature(tiles, rng),
            1 => self.add_columns(tiles, rng),
            2 => self.add_divider(tiles, rng),
            _ => {} // No additional feature
        }
    }
    
    // Add a central feature to a large hall
    fn add_central_feature(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rng: &mut impl Rng) {
        let center_x = self.x + self.width / 2;
        let center_y = self.y + self.height / 2;
        
        // Create a central feature (like an altar, statue, or fountain)
        let feature_size = rng.gen_range(1..=3);
        
        for y in center_y - feature_size / 2..=center_y + feature_size / 2 {
            for x in center_x - feature_size / 2..=center_x + feature_size / 2 {
                if x > 0 && x < MAP_WIDTH - 1 && y > 0 && y < MAP_HEIGHT - 1 {
                    tiles[y][x] = TileType::Wall;
                }
            }
        }
    }
    
    // Add columns to a large hall
    fn add_columns(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rng: &mut impl Rng) {
        // Calculate column positions
        let columns_per_row = (self.width / 4).max(2);
        let columns_per_col = (self.height / 4).max(2);
        
        let x_spacing = self.width / columns_per_row;
        let y_spacing = self.height / columns_per_col;
        
        // Place columns in a grid pattern
        for col_idx in 1..columns_per_row {
            for row_idx in 1..columns_per_col {
                let column_x = self.x + col_idx * x_spacing;
                let column_y = self.y + row_idx * y_spacing;
                
                // Add some randomness to column placement
                // Convert to i32 for the calculation, then back to usize
                let column_x_i32 = column_x as i32;
                let column_y_i32 = column_y as i32;
                let random_offset_x = rng.gen_range(-1..=1);
                let random_offset_y = rng.gen_range(-1..=1);
                
                let column_x = (column_x_i32 + random_offset_x) as usize;
                let column_y = (column_y_i32 + random_offset_y) as usize;
                
                // Ensure we're within bounds
                if column_x > 0 && column_x < MAP_WIDTH - 1 && column_y > 0 && column_y < MAP_HEIGHT - 1 {
                    tiles[column_y][column_x] = TileType::Wall;
                }
            }
        }
    }
    
    // Add a divider to create a more complex room
    fn add_divider(&self, tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rng: &mut impl Rng) {
        // Decide whether to add a horizontal or vertical divider
        let is_horizontal = self.width > self.height || (self.width == self.height && rng.gen_bool(0.5));
        
        if is_horizontal {
            // Add a horizontal divider with a gap
            let divider_y = self.y + self.height / 2;
            let gap_start = self.x + self.width / 3;
            let gap_end = self.x + 2 * self.width / 3;
            
            for x in self.x + 1..self.x + self.width - 1 {
                if x < gap_start || x > gap_end {
                    if divider_y < MAP_HEIGHT {
                        tiles[divider_y][x] = TileType::Wall;
                    }
                }
            }
        } else {
            // Add a vertical divider with a gap
            let divider_x = self.x + self.width / 2;
            let gap_start = self.y + self.height / 3;
            let gap_end = self.y + 2 * self.height / 3;
            
            for y in self.y + 1..self.y + self.height - 1 {
                if y < gap_start || y > gap_end {
                    if divider_x < MAP_WIDTH {
                        tiles[y][divider_x] = TileType::Wall;
                    }
                }
            }
        }
    }
}

impl TileMap {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut tiles = [[TileType::Wall; MAP_WIDTH]; MAP_HEIGHT];
        let mut biomes = [[BiomeType::Stone; MAP_WIDTH]; MAP_HEIGHT]; // Default biome
        
        // Generate rooms
        let rooms = Self::generate_rooms(&mut rng);
        
        // Carve out rooms
        for room in &rooms {
            room.carve(&mut tiles, &mut rng);
        }
        
        // Connect rooms with corridors
        Self::connect_rooms(&mut tiles, &rooms, &mut rng);
        
        // Add doors between rooms and corridors
        Self::add_doors(&mut tiles, &rooms, &mut rng);
        
        // Add secret rooms
        Self::add_secret_rooms(&mut tiles, &rooms, &mut rng);
        
        // Add extra corridors for more connectivity
        Self::add_extra_corridors(&mut tiles, &rooms, &mut rng);
        
        // Assign biomes to different regions of the map
        assign_biomes(&mut biomes, &rooms, &mut rng);
        
        // Find a valid spawn position (a floor tile)
        let spawn_position = Self::find_spawn_position(&tiles);
        
        Self {
            tiles,
            biomes,
            spawn_position,
        }
    }
    
    fn generate_rooms(rng: &mut impl Rng) -> Vec<Room> {
        let mut rooms = Vec::new();
        
        // Create a larger number of rooms with various sizes
        let num_rooms = rng.gen_range(20..30);
        
        // Track attempts to avoid infinite loops
        let mut attempts = 0;
        let max_attempts = 100;
        
        while rooms.len() < num_rooms && attempts < max_attempts {
            attempts += 1;
            
            // Determine room size category
            let size_category = match rng.gen_range(0..100) {
                0..=20 => RoomSize::Large,    // 21% chance for large rooms
                21..=60 => RoomSize::Medium,  // 40% chance for medium rooms
                _ => RoomSize::Small,         // 39% chance for small rooms
            };
            
            // Generate room dimensions based on size category
            let (room_width, room_height) = match size_category {
                RoomSize::Large => (
                    rng.gen_range(10..15),
                    rng.gen_range(10..15)
                ),
                RoomSize::Medium => (
                    rng.gen_range(6..10),
                    rng.gen_range(6..10)
                ),
                RoomSize::Small => (
                    rng.gen_range(3..6),
                    rng.gen_range(3..6)
                ),
            };
            
            // Generate random room position
            let room_x = rng.gen_range(1..MAP_WIDTH - room_width - 1);
            let room_y = rng.gen_range(1..MAP_HEIGHT - room_height - 1);
            
            // Choose a room type based on size
            let room_type = match size_category {
                RoomSize::Large => match rng.gen_range(0..100) {
                    0..=40 => RoomType::Rectangular,  // 41% chance
                    41..=60 => RoomType::Pillared,    // 20% chance
                    61..=80 => RoomType::CrossShaped, // 20% chance
                    _ => RoomType::LargeHall,         // 19% chance
                },
                RoomSize::Medium => match rng.gen_range(0..100) {
                    0..=30 => RoomType::Rectangular,  // 31% chance
                    31..=50 => RoomType::Circular,    // 20% chance
                    51..=70 => RoomType::LShaped,     // 20% chance
                    71..=90 => RoomType::Pillared,    // 20% chance
                    _ => RoomType::CrossShaped,       // 9% chance
                },
                RoomSize::Small => match rng.gen_range(0..100) {
                    0..=60 => RoomType::Rectangular,  // 61% chance
                    61..=90 => RoomType::Circular,    // 30% chance
                    _ => RoomType::SmallChamber,      // 9% chance
                },
            };
            
            let new_room = Room::new(room_x, room_y, room_width, room_height, room_type);
            
            // Check if the room overlaps with any existing room
            let mut has_overlap = false;
            for existing_room in &rooms {
                if new_room.overlaps(existing_room) {
                    has_overlap = true;
                    break;
                }
            }
            
            // If no overlap, add the room
            if !has_overlap {
                rooms.push(new_room);
            }
        }
        
        rooms
    }
    
    fn connect_rooms(tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rooms: &[Room], rng: &mut impl Rng) {
        if rooms.len() <= 1 {
            return;
        }
        
        // Create a list of all room connections
        let mut connections = Vec::new();
        
        // Connect each room to the next one
        for i in 0..rooms.len() - 1 {
            connections.push((i, i + 1));
        }
        
        // Connect the last room to the first to form a loop
        if rooms.len() > 2 {
            connections.push((rooms.len() - 1, 0));
        }
        
        // Add more random connections for more interesting layouts
        // Increase the number of extra connections based on room count
        let extra_connections = rooms.len() / 2;
        for _ in 0..extra_connections {
            let from = rng.gen_range(0..rooms.len());
            let mut to = rng.gen_range(0..rooms.len());
            
            // Ensure we don't connect a room to itself
            while from == to {
                to = rng.gen_range(0..rooms.len());
            }
            
            // Check if this connection already exists
            if !connections.contains(&(from, to)) && !connections.contains(&(to, from)) {
                connections.push((from, to));
            }
        }
        
        // Create corridors for all connections
        for (from, to) in connections {
            let (start_x, start_y) = rooms[from].center();
            let (end_x, end_y) = rooms[to].center();
            
            // Choose a corridor type based on distance, room types, and randomness
            let distance = ((start_x as i32 - end_x as i32).abs() + (start_y as i32 - end_y as i32).abs()) as usize;
            let from_room_size = rooms[from].size();
            let to_room_size = rooms[to].size();
            
            // Large rooms connected to large rooms get more complex corridors
            if (from_room_size == RoomSize::Large && to_room_size == RoomSize::Large) || distance > 20 || rng.gen_bool(0.4) {
                // For longer distances or between large rooms, use winding corridors with branches
                Self::create_branching_corridor(tiles, start_x, start_y, end_x, end_y, rng);
            } else if distance > 15 || rng.gen_bool(0.3) {
                // For medium distances, use winding corridors
                Self::create_winding_corridor(tiles, start_x, start_y, end_x, end_y, rng);
            } else if rng.gen_bool(0.5) {
                // Sometimes use Z-shaped corridors
                Self::create_z_corridor(tiles, start_x, start_y, end_x, end_y, rng);
            } else {
                // Otherwise use simple L-shaped corridors
                Self::create_corridor(tiles, start_x, start_y, end_x, end_y);
            }
            
            // Occasionally add a door at one end of the corridor
            if rng.gen_bool(0.4) {  // Increased chance for doors
                let door_pos = if rng.gen_bool(0.5) {
                    Self::find_door_position(tiles, start_x, start_y)
                } else {
                    Self::find_door_position(tiles, end_x, end_y)
                };
                
                if let Some((door_x, door_y)) = door_pos {
                    tiles[door_y][door_x] = TileType::Door;
                }
            }
        }
        
        // Add some standalone corridors that aren't connecting rooms
        Self::add_extra_corridors(tiles, rooms, rng);
    }
    
    fn find_door_position(tiles: &[[TileType; MAP_WIDTH]; MAP_HEIGHT], x: usize, y: usize) -> Option<(usize, usize)> {
        // Check all four adjacent tiles to find a suitable door position
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        
        for (dx, dy) in directions {
            let nx = (x as i32 + dx) as usize;
            let ny = (y as i32 + dy) as usize;
            
            // Ensure we're within bounds
            if nx > 0 && nx < MAP_WIDTH - 1 && ny > 0 && ny < MAP_HEIGHT - 1 {
                // Check if this position has a wall with floor on both sides
                if tiles[ny][nx] == TileType::Wall {
                    let opposite_x = (nx as i32 + dx) as usize;
                    let opposite_y = (ny as i32 + dy) as usize;
                    
                    if opposite_x > 0 && opposite_x < MAP_WIDTH - 1 && 
                       opposite_y > 0 && opposite_y < MAP_HEIGHT - 1 &&
                       tiles[opposite_y][opposite_x] == TileType::Floor {
                        return Some((nx, ny));
                    }
                }
            }
        }
        
        None
    }
    
    fn create_corridor(
        tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT],
        start_x: usize, start_y: usize,
        end_x: usize, end_y: usize
    ) {
        // Create a simple L-shaped corridor
        // First horizontal, then vertical
        Self::create_horizontal_corridor(tiles, start_x, end_x, start_y);
        Self::create_vertical_corridor(tiles, start_y, end_y, end_x);
    }
    
    fn create_z_corridor(
        tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT],
        start_x: usize, start_y: usize,
        end_x: usize, end_y: usize,
        rng: &mut impl Rng
    ) {
        // Create a Z-shaped corridor with a middle segment
        let mid_x = if start_x < end_x {
            start_x + (end_x - start_x) / 2
        } else {
            end_x + (start_x - end_x) / 2
        };
        
        // Add some randomness to the middle point
        let mid_x = if mid_x > 5 && mid_x < MAP_WIDTH - 5 {
            // Convert to i32 for the calculation, then back to usize
            let mid_x_i32 = mid_x as i32;
            let random_offset = rng.gen_range(-3..=3);
            (mid_x_i32 + random_offset) as usize
        } else {
            mid_x
        };
        
        // Create the three segments of the Z
        Self::create_horizontal_corridor(tiles, start_x, mid_x, start_y);
        Self::create_vertical_corridor(tiles, start_y, end_y, mid_x);
        Self::create_horizontal_corridor(tiles, mid_x, end_x, end_y);
    }
    
    fn create_winding_corridor(
        tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT],
        start_x: usize, start_y: usize,
        end_x: usize, end_y: usize,
        rng: &mut impl Rng
    ) {
        // Create a winding corridor with multiple segments
        let mut current_x = start_x;
        let mut current_y = start_y;
        
        // Determine number of segments based on distance
        let distance = ((start_x as i32 - end_x as i32).abs() + (start_y as i32 - end_y as i32).abs()) as usize;
        let num_segments = (distance / 5).max(2).min(5);
        
        for _ in 0..num_segments {
            // Choose whether to move horizontally or vertically
            if rng.gen_bool(0.5) {
                // Move horizontally towards the target
                let target_x = if current_x < end_x {
                    current_x + (end_x - current_x) / 2
                } else {
                    current_x - (current_x - end_x) / 2
                };
                
                // Add some randomness
                let target_x = if target_x > 5 && target_x < MAP_WIDTH - 5 {
                    // Convert to i32 for the calculation, then back to usize
                    let target_x_i32 = target_x as i32;
                    let random_offset = rng.gen_range(-2..=2);
                    (target_x_i32 + random_offset) as usize
                } else {
                    target_x
                };
                
                Self::create_horizontal_corridor(tiles, current_x, target_x, current_y);
                current_x = target_x;
            } else {
                // Move vertically towards the target
                let target_y = if current_y < end_y {
                    current_y + (end_y - current_y) / 2
                } else {
                    current_y - (current_y - end_y) / 2
                };
                
                // Add some randomness
                let target_y = if target_y > 5 && target_y < MAP_HEIGHT - 5 {
                    // Convert to i32 for the calculation, then back to usize
                    let target_y_i32 = target_y as i32;
                    let random_offset = rng.gen_range(-2..=2);
                    (target_y_i32 + random_offset) as usize
                } else {
                    target_y
                };
                
                Self::create_vertical_corridor(tiles, current_y, target_y, current_x);
                current_y = target_y;
            }
        }
        
        // Final segment to reach the destination
        Self::create_horizontal_corridor(tiles, current_x, end_x, current_y);
        Self::create_vertical_corridor(tiles, current_y, end_y, end_x);
    }
    
    fn create_horizontal_corridor(
        tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT],
        x1: usize, x2: usize, y: usize
    ) {
        let start = x1.min(x2);
        let end = x1.max(x2);
        
        for x in start..=end {
            if x > 0 && x < MAP_WIDTH - 1 && y > 0 && y < MAP_HEIGHT - 1 {
                tiles[y][x] = TileType::Floor;
            }
        }
    }
    
    fn create_vertical_corridor(
        tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT],
        y1: usize, y2: usize, x: usize
    ) {
        let start = y1.min(y2);
        let end = y1.max(y2);
        
        for y in start..=end {
            if x > 0 && x < MAP_WIDTH - 1 && y > 0 && y < MAP_HEIGHT - 1 {
                tiles[y][x] = TileType::Floor;
            }
        }
    }
    
    fn add_secret_rooms(tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rooms: &[Room], rng: &mut impl Rng) {
        // Try to add 1-3 secret rooms
        let num_secret_rooms = rng.gen_range(1..=3);
        
        for _ in 0..num_secret_rooms {
            // Find a suitable wall location for a secret room
            let mut attempts = 0;
            let max_attempts = 50;
            
            while attempts < max_attempts {
                // Choose a random position on the map
                let x = rng.gen_range(3..MAP_WIDTH - 6);
                let y = rng.gen_range(3..MAP_HEIGHT - 6);
                
                // Check if this is a wall with at least one adjacent floor tile
                if tiles[y][x] == TileType::Wall && Self::has_adjacent_floor(tiles, x, y) {
                    // Create a small secret room
                    let room_width = rng.gen_range(3..6);
                    let room_height = rng.gen_range(3..6);
                    
                    // Check if there's enough space for the room
                    let mut can_place = true;
                    for ry in y..y + room_height {
                        for rx in x..x + room_width {
                            if rx >= MAP_WIDTH || ry >= MAP_HEIGHT || tiles[ry][rx] == TileType::Floor {
                                can_place = false;
                                break;
                            }
                        }
                        if !can_place {
                            break;
                        }
                    }
                    
                    if can_place {
                        // Carve the secret room
                        for ry in y..y + room_height {
                            for rx in x..x + room_width {
                                if rx < MAP_WIDTH && ry < MAP_HEIGHT {
                                    tiles[ry][rx] = TileType::Floor;
                                }
                            }
                        }
                        
                        // Add a secret door
                        tiles[y][x] = TileType::SecretDoor;
                        
                        // Maybe add a special feature in the secret room
                        if rng.gen_bool(0.5) {
                            let feature_x = x + room_width / 2;
                            let feature_y = y + room_height / 2;
                            
                            if feature_x < MAP_WIDTH && feature_y < MAP_HEIGHT {
                                // For now, just add a pillar as a placeholder for a special feature
                                tiles[feature_y][feature_x] = TileType::Wall;
                            }
                        }
                        
                        break;
                    }
                }
                
                attempts += 1;
            }
        }
    }
    
    fn has_adjacent_floor(tiles: &[[TileType; MAP_WIDTH]; MAP_HEIGHT], x: usize, y: usize) -> bool {
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        
        for (dx, dy) in directions {
            let nx = (x as i32 + dx) as usize;
            let ny = (y as i32 + dy) as usize;
            
            if nx < MAP_WIDTH && ny < MAP_HEIGHT && tiles[ny][nx] == TileType::Floor {
                return true;
            }
        }
        
        false
    }
    
    fn find_spawn_position(tiles: &[[TileType; MAP_WIDTH]; MAP_HEIGHT]) -> (usize, usize) {
        // Find a valid floor tile to spawn the player
        let mut floor_tiles = Vec::new();
        
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                if tiles[y][x] == TileType::Floor {
                    floor_tiles.push((x, y));
                }
            }
        }
        
        if !floor_tiles.is_empty() {
            // Choose a random floor tile
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..floor_tiles.len());
            floor_tiles[index]
        } else {
            // Fallback to center of map if no floor tiles
            (MAP_WIDTH / 2, MAP_HEIGHT / 2)
        }
    }

    pub fn get_spawn_position(&self) -> (usize, usize) {
        self.spawn_position
    }

    fn add_extra_corridors(tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rooms: &[Room], rng: &mut impl Rng) {
        // Add 2-4 extra corridors that aren't directly connecting rooms
        let num_extra_corridors = rng.gen_range(2..=4);
        
        for _ in 0..num_extra_corridors {
            // Choose a random starting point from an existing floor tile
            let mut floor_tiles = Vec::new();
            
            for y in 1..MAP_HEIGHT-1 {
                for x in 1..MAP_WIDTH-1 {
                    if tiles[y][x] == TileType::Floor {
                        // Check if this is near a wall (corridor or room edge)
                        let has_adjacent_wall = 
                            tiles[y-1][x] == TileType::Wall || 
                            tiles[y+1][x] == TileType::Wall || 
                            tiles[y][x-1] == TileType::Wall || 
                            tiles[y][x+1] == TileType::Wall;
                        
                        if has_adjacent_wall {
                            floor_tiles.push((x, y));
                        }
                    }
                }
            }
            
            if floor_tiles.is_empty() {
                continue;
            }
            
            // Choose a random starting point
            let start_idx = rng.gen_range(0..floor_tiles.len());
            let (start_x, start_y) = floor_tiles[start_idx];
            
            // Generate a random corridor length and direction
            let length = rng.gen_range(5..15);
            let direction = match rng.gen_range(0..4) {
                0 => (1, 0),   // Right
                1 => (-1, 0),  // Left
                2 => (0, 1),   // Down
                _ => (0, -1),  // Up
            };
            
            // Create the corridor
            let mut current_x = start_x as i32;
            let mut current_y = start_y as i32;
            
            for _ in 0..length {
                current_x += direction.0;
                current_y += direction.1;
                
                // Ensure we're within bounds
                if current_x <= 0 || current_x >= MAP_WIDTH as i32 - 1 || 
                   current_y <= 0 || current_y >= MAP_HEIGHT as i32 - 1 {
                    break;
                }
                
                // Carve the corridor
                tiles[current_y as usize][current_x as usize] = TileType::Floor;
                
                // Occasionally branch off
                if rng.gen_bool(0.2) {
                    let branch_direction = match rng.gen_range(0..2) {
                        0 => (direction.1, direction.0),  // Perpendicular
                        _ => (-direction.1, -direction.0), // Other perpendicular
                    };
                    
                    let branch_length = rng.gen_range(3..8);
                    let mut branch_x = current_x;
                    let mut branch_y = current_y;
                    
                    for _ in 0..branch_length {
                        branch_x += branch_direction.0;
                        branch_y += branch_direction.1;
                        
                        // Ensure we're within bounds
                        if branch_x <= 0 || branch_x >= MAP_WIDTH as i32 - 1 || 
                           branch_y <= 0 || branch_y >= MAP_HEIGHT as i32 - 1 {
                            break;
                        }
                        
                        // Carve the branch
                        tiles[branch_y as usize][branch_x as usize] = TileType::Floor;
                    }
                }
            }
        }
    }
    
    fn create_branching_corridor(
        tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT],
        start_x: usize, start_y: usize,
        end_x: usize, end_y: usize,
        rng: &mut impl Rng
    ) {
        // Create a winding corridor with branches
        let mut current_x = start_x;
        let mut current_y = start_y;
        
        // Determine number of segments based on distance
        let distance = ((start_x as i32 - end_x as i32).abs() + (start_y as i32 - end_y as i32).abs()) as usize;
        let num_segments = (distance / 4).max(3).min(6);
        
        // Track corridor points for potential branches
        let mut corridor_points = Vec::new();
        corridor_points.push((current_x, current_y));
        
        for _ in 0..num_segments {
            // Choose whether to move horizontally or vertically
            if rng.gen_bool(0.5) {
                // Move horizontally towards the target
                let target_x = if current_x < end_x {
                    current_x + (end_x - current_x) / 2
                } else {
                    current_x - (current_x - end_x) / 2
                };
                
                // Add some randomness
                let target_x = if target_x > 5 && target_x < MAP_WIDTH - 5 {
                    // Convert to i32 for the calculation, then back to usize
                    let target_x_i32 = target_x as i32;
                    let random_offset = rng.gen_range(-3..=3);
                    (target_x_i32 + random_offset) as usize
                } else {
                    target_x
                };
                
                Self::create_horizontal_corridor(tiles, current_x, target_x, current_y);
                current_x = target_x;
            } else {
                // Move vertically towards the target
                let target_y = if current_y < end_y {
                    current_y + (end_y - current_y) / 2
                } else {
                    current_y - (current_y - end_y) / 2
                };
                
                // Add some randomness
                let target_y = if target_y > 5 && target_y < MAP_HEIGHT - 5 {
                    // Convert to i32 for the calculation, then back to usize
                    let target_y_i32 = target_y as i32;
                    let random_offset = rng.gen_range(-3..=3);
                    (target_y_i32 + random_offset) as usize
                } else {
                    target_y
                };
                
                Self::create_vertical_corridor(tiles, current_y, target_y, current_x);
                current_y = target_y;
            }
            
            // Add this point to potential branch locations
            corridor_points.push((current_x, current_y));
            
            // Occasionally add a corridor feature
            if rng.gen_bool(0.2) {
                Self::add_corridor_feature(tiles, current_x, current_y, rng);
            }
        }
        
        // Final segment to reach the destination
        Self::create_horizontal_corridor(tiles, current_x, end_x, current_y);
        Self::create_vertical_corridor(tiles, current_y, end_y, end_x);
        
        // Add branches from the main corridor
        let num_branches = rng.gen_range(1..=3);
        for _ in 0..num_branches {
            if corridor_points.len() < 2 {
                break;
            }
            
            // Choose a random point along the corridor (not the start or end)
            let branch_idx = rng.gen_range(1..corridor_points.len() - 1);
            let (branch_x, branch_y) = corridor_points[branch_idx];
            
            // Choose a random direction and length for the branch
            let direction = match rng.gen_range(0..4) {
                0 => (1, 0),   // Right
                1 => (-1, 0),  // Left
                2 => (0, 1),   // Down
                _ => (0, -1),  // Up
            };
            
            let branch_length = rng.gen_range(3..8);
            let mut current_x = branch_x as i32;
            let mut current_y = branch_y as i32;
            
            for _ in 0..branch_length {
                current_x += direction.0;
                current_y += direction.1;
                
                // Ensure we're within bounds
                if current_x <= 0 || current_x >= MAP_WIDTH as i32 - 1 || 
                   current_y <= 0 || current_y >= MAP_HEIGHT as i32 - 1 {
                    break;
                }
                
                // Carve the branch
                tiles[current_y as usize][current_x as usize] = TileType::Floor;
            }
        }
    }
    
    fn add_corridor_feature(
        tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT],
        x: usize, y: usize,
        rng: &mut impl Rng
    ) {
        // Choose a feature type
        match rng.gen_range(0..3) {
            0 => Self::add_corridor_alcove(tiles, x, y, rng),
            1 => Self::add_corridor_pillar(tiles, x, y),
            _ => Self::add_corridor_widening(tiles, x, y, rng),
        }
    }
    
    fn add_corridor_alcove(
        tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT],
        x: usize, y: usize,
        rng: &mut impl Rng
    ) {
        // Create a small alcove off the corridor
        let direction = match rng.gen_range(0..4) {
            0 => (1, 0),   // Right
            1 => (-1, 0),  // Left
            2 => (0, 1),   // Down
            _ => (0, -1),  // Up
        };
        
        let alcove_size = rng.gen_range(1..=3);
        
        for i in 1..=alcove_size {
            let nx = (x as i32 + direction.0 * i) as usize;
            let ny = (y as i32 + direction.1 * i) as usize;
            
            // Ensure we're within bounds
            if nx <= 0 || nx >= MAP_WIDTH - 1 || ny <= 0 || ny >= MAP_HEIGHT - 1 {
                break;
            }
            
            // Carve the alcove
            tiles[ny][nx] = TileType::Floor;
            
            // Add side tiles for wider alcoves
            if i > 1 {
                let side_dir = (direction.1, direction.0); // Perpendicular
                
                for j in -1..=1 {
                    if j == 0 {
                        continue; // Skip the center tile
                    }
                    
                    let sx = (nx as i32 + side_dir.0 * j) as usize;
                    let sy = (ny as i32 + side_dir.1 * j) as usize;
                    
                    // Ensure we're within bounds
                    if sx <= 0 || sx >= MAP_WIDTH - 1 || sy <= 0 || sy >= MAP_HEIGHT - 1 {
                        continue;
                    }
                    
                    // Carve the side tile
                    tiles[sy][sx] = TileType::Floor;
                }
            }
        }
    }
    
    fn add_corridor_pillar(
        tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT],
        x: usize, y: usize
    ) {
        // Check if there's enough space for a pillar
        if x <= 1 || x >= MAP_WIDTH - 2 || y <= 1 || y >= MAP_HEIGHT - 2 {
            return;
        }
        
        // Check if we're in a wider area
        let has_space = 
            tiles[y-1][x-1] == TileType::Floor && 
            tiles[y-1][x+1] == TileType::Floor && 
            tiles[y+1][x-1] == TileType::Floor && 
            tiles[y+1][x+1] == TileType::Floor;
        
        if has_space {
            // Add a pillar
            tiles[y][x] = TileType::Wall;
        }
    }
    
    fn add_corridor_widening(
        tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT],
        x: usize, y: usize,
        rng: &mut impl Rng
    ) {
        // Widen the corridor in all directions
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue; // Skip the center tile
                }
                
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                
                // Ensure we're within bounds
                if nx <= 0 || nx >= MAP_WIDTH - 1 || ny <= 0 || ny >= MAP_HEIGHT - 1 {
                    continue;
                }
                
                // Randomly decide whether to carve this tile
                if rng.gen_bool(0.7) {
                    tiles[ny][nx] = TileType::Floor;
                }
            }
        }
    }

    // Get the biome at a specific position
    pub fn get_biome_at(&self, x: usize, y: usize) -> BiomeType {
        if x < MAP_WIDTH && y < MAP_HEIGHT {
            self.biomes[y][x]
        } else {
            BiomeType::Stone // Default biome
        }
    }

    fn add_doors(tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], rooms: &[Room], rng: &mut impl Rng) {
        // Add doors between rooms and corridors
        for room in rooms {
            // Try to add doors on each side of the room
            // Top side
            for x in room.x + 1..room.x + room.width - 1 {
                if x < MAP_WIDTH - 1 && room.y > 0 {
                    // Check if there's a wall with floor on both sides
                    if tiles[room.y][x] == TileType::Wall &&
                       tiles[room.y - 1][x] == TileType::Floor &&
                       tiles[room.y + 1][x] == TileType::Floor {
                        // 30% chance to add a door
                        if rng.gen_bool(0.3) {
                            tiles[room.y][x] = TileType::Door;
                        }
                    }
                }
            }
            
            // Bottom side
            for x in room.x + 1..room.x + room.width - 1 {
                if x < MAP_WIDTH - 1 && room.y + room.height < MAP_HEIGHT - 1 {
                    // Check if there's a wall with floor on both sides
                    if tiles[room.y + room.height - 1][x] == TileType::Wall &&
                       tiles[room.y + room.height - 2][x] == TileType::Floor &&
                       tiles[room.y + room.height][x] == TileType::Floor {
                        // 30% chance to add a door
                        if rng.gen_bool(0.3) {
                            tiles[room.y + room.height - 1][x] = TileType::Door;
                        }
                    }
                }
            }
            
            // Left side
            for y in room.y + 1..room.y + room.height - 1 {
                if y < MAP_HEIGHT - 1 && room.x > 0 {
                    // Check if there's a wall with floor on both sides
                    if tiles[y][room.x] == TileType::Wall &&
                       tiles[y][room.x - 1] == TileType::Floor &&
                       tiles[y][room.x + 1] == TileType::Floor {
                        // 30% chance to add a door
                        if rng.gen_bool(0.3) {
                            tiles[y][room.x] = TileType::Door;
                        }
                    }
                }
            }
            
            // Right side
            for y in room.y + 1..room.y + room.height - 1 {
                if y < MAP_HEIGHT - 1 && room.x + room.width < MAP_WIDTH - 1 {
                    // Check if there's a wall with floor on both sides
                    if tiles[y][room.x + room.width - 1] == TileType::Wall &&
                       tiles[y][room.x + room.width - 2] == TileType::Floor &&
                       tiles[y][room.x + room.width] == TileType::Floor {
                        // 30% chance to add a door
                        if rng.gen_bool(0.3) {
                            tiles[y][room.x + room.width - 1] = TileType::Door;
                        }
                    }
                }
            }
        }
    }
}

// Assign biomes to different regions of the map
fn assign_biomes(biomes: &mut [[BiomeType; MAP_WIDTH]; MAP_HEIGHT], rooms: &[Room], rng: &mut impl Rng) {
    // Decide if the map will have 1 or 2 biomes
    let biome_count = if rng.gen_bool(0.3) { 1 } else { 2 };
    
    // Select the biomes for this map
    let available_biomes = [
        BiomeType::Stone,
        BiomeType::Dirt,
        BiomeType::Catacombs,
        BiomeType::Grass,
        BiomeType::Igneous,
    ];
    
    // Randomly select 1 or 2 biomes
    let mut map_biomes = Vec::with_capacity(biome_count);
    
    // Always include at least one biome
    let first_biome = available_biomes[rng.gen_range(0..available_biomes.len())];
    map_biomes.push(first_biome);
    
    // Add a second biome if needed, ensuring it's different from the first
    if biome_count == 2 {
        let mut second_biome;
        loop {
            second_biome = available_biomes[rng.gen_range(0..available_biomes.len())];
            if second_biome != first_biome {
                break;
            }
        }
        map_biomes.push(second_biome);
    }
    
    println!("Map generated with {} biome(s): {:?}", biome_count, map_biomes);
    
    // Assign biomes to rooms
    for (i, room) in rooms.iter().enumerate() {
        // Choose a biome for this room from our selected biomes
        let biome_index = if biome_count == 1 {
            0 // Only one biome available
        } else {
            // Alternate biomes or choose randomly
            // Using modulo to create patterns of biome distribution
            i % biome_count
        };
        
        let biome = map_biomes[biome_index];
        
        // Apply the biome to the room area
        for y in room.y..(room.y + room.height) {
            for x in room.x..(room.x + room.width) {
                if y < MAP_HEIGHT && x < MAP_WIDTH {
                    biomes[y][x] = biome;
                }
            }
        }
    }
    
    // Create some larger biome regions by expanding from rooms
    let mut visited = [[false; MAP_WIDTH]; MAP_HEIGHT];
    
    // Mark room areas as visited
    for room in rooms {
        for y in room.y..(room.y + room.height) {
            for x in room.x..(room.x + room.width) {
                if y < MAP_HEIGHT && x < MAP_WIDTH {
                    visited[y][x] = true;
                }
            }
        }
    }
    
    // Expand biomes to nearby areas
    for _ in 0..3 { // Multiple passes to expand further
        let mut new_visited = visited.clone();
        
        for y in 1..MAP_HEIGHT-1 {
            for x in 1..MAP_WIDTH-1 {
                if !visited[y][x] {
                    // Check neighbors
                    let mut neighbor_biomes = Vec::new();
                    
                    if visited[y-1][x] {
                        neighbor_biomes.push(biomes[y-1][x]);
                    }
                    if visited[y+1][x] {
                        neighbor_biomes.push(biomes[y+1][x]);
                    }
                    if visited[y][x-1] {
                        neighbor_biomes.push(biomes[y][x-1]);
                    }
                    if visited[y][x+1] {
                        neighbor_biomes.push(biomes[y][x+1]);
                    }
                    
                    // If we have neighbors with biomes, choose one
                    if !neighbor_biomes.is_empty() {
                        let chosen_biome = *neighbor_biomes.choose(rng).unwrap();
                        biomes[y][x] = chosen_biome;
                        new_visited[y][x] = true;
                    }
                }
            }
        }
        
        visited = new_visited;
    }
    
    // Fill any remaining areas with the default biome (first selected biome)
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            if !visited[y][x] {
                biomes[y][x] = map_biomes[0];
            }
        }
    }
}
