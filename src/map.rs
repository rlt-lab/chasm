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

impl TileMap {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut tiles = [[TileType::Wall; MAP_WIDTH]; MAP_HEIGHT];
        
        // Step 1: Initialize the map with random walls and floors
        // We'll use a higher probability of walls to create more cave-like structures
        for y in 1..MAP_HEIGHT-1 {
            for x in 1..MAP_WIDTH-1 {
                // 45% chance of being floor initially
                if rng.gen_bool(0.45) {
                    tiles[y][x] = TileType::Floor;
                }
            }
        }
        
        // Step 2: Apply cellular automata rules for several iterations
        let iterations = 5;
        for _ in 0..iterations {
            tiles = Self::apply_cellular_automata_step(tiles);
        }
        
        // Step 3: Ensure the map is connected (find and connect disconnected regions)
        tiles = Self::connect_regions(tiles, &mut rng);
        
        // Step 4: Find a suitable spawn position (a floor tile not too close to walls)
        let spawn_position = Self::find_spawn_position(&tiles, &mut rng);
        
        TileMap { tiles, spawn_position }
    }
    
    fn apply_cellular_automata_step(tiles: [[TileType; MAP_WIDTH]; MAP_HEIGHT]) -> [[TileType; MAP_WIDTH]; MAP_HEIGHT] {
        let mut new_tiles = [[TileType::Wall; MAP_WIDTH]; MAP_HEIGHT];
        
        // Apply cellular automata rules:
        // 1. A floor tile with 4 or more wall neighbors becomes a wall
        // 2. A wall tile with 3 or fewer wall neighbors becomes a floor
        for y in 1..MAP_HEIGHT-1 {
            for x in 1..MAP_WIDTH-1 {
                let wall_neighbors = Self::count_wall_neighbors(x, y, &tiles);
                
                if tiles[y][x] == TileType::Floor {
                    // Rule 1: Floor becomes wall if too many wall neighbors
                    if wall_neighbors >= 5 {
                        new_tiles[y][x] = TileType::Wall;
                    } else {
                        new_tiles[y][x] = TileType::Floor;
                    }
                } else {
                    // Rule 2: Wall becomes floor if too few wall neighbors
                    if wall_neighbors <= 3 {
                        new_tiles[y][x] = TileType::Floor;
                    } else {
                        new_tiles[y][x] = TileType::Wall;
                    }
                }
            }
        }
        
        // Ensure the borders are always walls
        for y in 0..MAP_HEIGHT {
            new_tiles[y][0] = TileType::Wall;
            new_tiles[y][MAP_WIDTH-1] = TileType::Wall;
        }
        for x in 0..MAP_WIDTH {
            new_tiles[0][x] = TileType::Wall;
            new_tiles[MAP_HEIGHT-1][x] = TileType::Wall;
        }
        
        new_tiles
    }
    
    fn count_wall_neighbors(x: usize, y: usize, tiles: &[[TileType; MAP_WIDTH]; MAP_HEIGHT]) -> usize {
        let mut count = 0;
        
        // Check all 8 neighboring cells
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue; // Skip the center cell
                }
                
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                // Count out-of-bounds as walls
                if nx < 0 || nx >= MAP_WIDTH as i32 || ny < 0 || ny >= MAP_HEIGHT as i32 {
                    count += 1;
                } else if tiles[ny as usize][nx as usize] == TileType::Wall {
                    count += 1;
                }
            }
        }
        
        count
    }
    
    fn connect_regions(mut tiles: [[TileType; MAP_WIDTH]; MAP_HEIGHT], _rng: &mut impl Rng) -> [[TileType; MAP_WIDTH]; MAP_HEIGHT] {
        // Find all separate regions
        let regions = Self::find_regions(&tiles);
        
        // If there's only one region, we're done
        if regions.len() <= 1 {
            return tiles;
        }
        
        // Sort regions by size (largest first)
        let mut sorted_regions = regions.clone();
        sorted_regions.sort_by(|a, b| b.len().cmp(&a.len()));
        
        // The largest region is our main region
        let main_region = &sorted_regions[0];
        
        // Connect each other region to the main region
        for i in 1..sorted_regions.len() {
            let region = &sorted_regions[i];
            
            // Find the closest points between the main region and this region
            if let Some((main_point, region_point)) = Self::find_closest_points(main_region, region) {
                // Create a corridor between these points
                Self::create_corridor(&mut tiles, main_point.0, main_point.1, region_point.0, region_point.1);
            }
        }
        
        tiles
    }
    
    fn find_regions(tiles: &[[TileType; MAP_WIDTH]; MAP_HEIGHT]) -> Vec<Vec<(usize, usize)>> {
        let mut regions = Vec::new();
        let mut visited = [[false; MAP_WIDTH]; MAP_HEIGHT];
        
        for y in 1..MAP_HEIGHT-1 {
            for x in 1..MAP_WIDTH-1 {
                if tiles[y][x] == TileType::Floor && !visited[y][x] {
                    // Found a new region, flood fill to find all connected tiles
                    let mut region = Vec::new();
                    let mut queue = vec![(x, y)];
                    visited[y][x] = true;
                    
                    while let Some((cx, cy)) = queue.pop() {
                        region.push((cx, cy));
                        
                        // Check all 4 adjacent tiles
                        for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)].iter() {
                            let nx = (cx as i32 + dx) as usize;
                            let ny = (cy as i32 + dy) as usize;
                            
                            if nx > 0 && nx < MAP_WIDTH-1 && ny > 0 && ny < MAP_HEIGHT-1 &&
                               tiles[ny][nx] == TileType::Floor && !visited[ny][nx] {
                                visited[ny][nx] = true;
                                queue.push((nx, ny));
                            }
                        }
                    }
                    
                    regions.push(region);
                }
            }
        }
        
        regions
    }
    
    fn find_closest_points(region1: &[(usize, usize)], region2: &[(usize, usize)]) -> Option<((usize, usize), (usize, usize))> {
        let mut min_distance = f32::MAX;
        let mut closest_pair = None;
        
        for &p1 in region1 {
            for &p2 in region2 {
                let dx = p1.0 as i32 - p2.0 as i32;
                let dy = p1.1 as i32 - p2.1 as i32;
                let distance = (dx * dx + dy * dy) as f32;
                
                if distance < min_distance {
                    min_distance = distance;
                    closest_pair = Some((p1, p2));
                }
            }
        }
        
        closest_pair
    }
    
    fn create_corridor(tiles: &mut [[TileType; MAP_WIDTH]; MAP_HEIGHT], 
                      start_x: usize, start_y: usize, 
                      end_x: usize, end_y: usize) {
        // Draw horizontal corridor
        let xstart = start_x.min(end_x);
        let xend = start_x.max(end_x);
        for x in xstart..=xend {
            tiles[start_y][x] = TileType::Floor;
        }

        // Draw vertical corridor
        let ystart = start_y.min(end_y);
        let yend = start_y.max(end_y);
        for y in ystart..=yend {
            tiles[y][end_x] = TileType::Floor;
        }
    }
    
    fn find_spawn_position(tiles: &[[TileType; MAP_WIDTH]; MAP_HEIGHT], rng: &mut impl Rng) -> (usize, usize) {
        // Collect all floor tiles
        let mut floor_tiles = Vec::new();
        
        for y in 1..MAP_HEIGHT-1 {
            for x in 1..MAP_WIDTH-1 {
                if tiles[y][x] == TileType::Floor {
                    // Check if this is a good spawn position (not too close to walls)
                    let mut wall_neighbors = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            
                            if nx >= 0 && nx < MAP_WIDTH as i32 && ny >= 0 && ny < MAP_HEIGHT as i32 &&
                               tiles[ny as usize][nx as usize] == TileType::Wall {
                                wall_neighbors += 1;
                            }
                        }
                    }
                    
                    // Only consider tiles with few wall neighbors
                    if wall_neighbors <= 2 {
                        floor_tiles.push((x, y));
                    }
                }
            }
        }
        
        // If we found suitable tiles, pick one randomly
        if !floor_tiles.is_empty() {
            let index = rng.gen_range(0..floor_tiles.len());
            return floor_tiles[index];
        }
        
        // Fallback: just pick any floor tile
        for y in 1..MAP_HEIGHT-1 {
            for x in 1..MAP_WIDTH-1 {
                if tiles[y][x] == TileType::Floor {
                    return (x, y);
                }
            }
        }
        
        // Ultimate fallback
        (MAP_WIDTH / 2, MAP_HEIGHT / 2)
    }

    pub fn get_spawn_position(&self) -> (usize, usize) {
        self.spawn_position
    }
}

pub fn spawn_map(commands: &mut Commands) -> Entity {
    commands.spawn(TileMap::new()).id()
}

