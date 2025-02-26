use bevy::prelude::*;
use std::collections::HashMap;
use rand::Rng;
use rand::rngs::StdRng;

/// Represents different biome types in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeType {
    Caves,      // Cave areas with dirt and stone walls
    Groves,     // Overgrown areas with grass and plants
    Labyrinth,  // Maze-like areas with stone brick walls
    Catacombs,  // Areas with skull walls and bone floors
}

/// Represents the walkability status of a tile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileWalkability {
    Walkable,
    Blocked,
    Door,       // Special case - can be walked through but requires interaction
}

/// Stores information about a specific tile type
#[derive(Debug, Clone)]
pub struct TileInfo {
    pub name: String,
    pub sprite_index: usize,
    pub walkability: TileWalkability,
    pub biome: BiomeType,
    pub color: Color,
}

/// Resource that manages biome-specific tile information
#[derive(Resource)]
pub struct BiomeManager {
    pub biome_tiles: HashMap<BiomeType, Vec<TileInfo>>,
    pub walkable_tiles: Vec<TileInfo>,
    pub wall_tiles: Vec<TileInfo>,
    pub door_tiles: Vec<TileInfo>,
}

impl Default for BiomeManager {
    fn default() -> Self {
        Self {
            biome_tiles: HashMap::new(),
            walkable_tiles: Vec::new(),
            wall_tiles: Vec::new(),
            door_tiles: Vec::new(),
        }
    }
}

impl BiomeManager {
    /// Register a tile with its properties
    pub fn register_tile(&mut self, name: &str, sprite_index: usize, walkability: TileWalkability, biome: BiomeType) {
        // Determine the appropriate color based on the biome or tile name
        let color = match biome {
            BiomeType::Caves => Color::rgb(0.5, 0.5, 0.5), // Grey for caves
            BiomeType::Groves => Color::rgb(0.3, 0.7, 0.3), // Green for groves
            BiomeType::Labyrinth => Color::rgb(0.5, 0.3, 0.5), // Purple for labyrinth
            BiomeType::Catacombs => Color::rgb(0.5, 0.3, 0.5), // Purple for catacombs
        };

        let tile_info = TileInfo {
            name: name.to_string(),
            sprite_index,
            walkability,
            biome,
            color,
        };
        
        // Add to biome-specific collection
        self.biome_tiles.entry(biome)
            .or_insert_with(Vec::new)
            .push(tile_info.clone());
            
        // Also add to walkability collections for quick access
        match walkability {
            TileWalkability::Walkable => self.walkable_tiles.push(tile_info),
            TileWalkability::Blocked => self.wall_tiles.push(tile_info),
            TileWalkability::Door => self.door_tiles.push(tile_info),
        }
    }
    
    /// Get a random walkable tile for a specific biome
    pub fn get_random_floor_tile(&self, biome: BiomeType, rng: &mut impl Rng) -> Option<&TileInfo> {
        let biome_tiles = self.biome_tiles.get(&biome)?;
        let walkable_tiles: Vec<&TileInfo> = biome_tiles.iter()
            .filter(|tile| tile.walkability == TileWalkability::Walkable)
            .collect();
            
        if walkable_tiles.is_empty() {
            return None;
        }
        
        let index = rng.gen_range(0..walkable_tiles.len());
        Some(walkable_tiles[index])
    }
    
    /// Get a random wall tile for a specific biome
    pub fn get_random_wall_tile(&self, biome: BiomeType, rng: &mut impl Rng) -> Option<&TileInfo> {
        let biome_tiles = self.biome_tiles.get(&biome)?;
        let wall_tiles: Vec<&TileInfo> = biome_tiles.iter()
            .filter(|tile| tile.walkability == TileWalkability::Blocked)
            .collect();
            
        if wall_tiles.is_empty() {
            return None;
        }
        
        let index = rng.gen_range(0..wall_tiles.len());
        Some(wall_tiles[index])
    }
    
    /// Get a wall tile based on its position in the map
    /// This function selects between top and side wall tiles based on context
    pub fn get_wall_tile_for_position(
        &self, 
        biome: BiomeType, 
        x: usize, 
        y: usize, 
        map: &crate::map::TileMap,
        rng: &mut impl Rng
    ) -> Option<&TileInfo> {
        let biome_tiles = self.biome_tiles.get(&biome)?;
        
        // Filter wall tiles by type (top or side)
        let top_wall_tiles: Vec<&TileInfo> = biome_tiles.iter()
            .filter(|tile| 
                tile.walkability == TileWalkability::Blocked && 
                tile.name.contains("(top)")
            )
            .collect();
            
        let side_wall_tiles: Vec<&TileInfo> = biome_tiles.iter()
            .filter(|tile| 
                tile.walkability == TileWalkability::Blocked && 
                tile.name.contains("(side)")
            )
            .collect();
        
        if top_wall_tiles.is_empty() && side_wall_tiles.is_empty() {
            return None;
        }
        
        // PRIMARY RULE: If this wall is directly above a floor tile, use a side wall tile
        let is_above_floor = y > 0 && map.tiles[y-1][x] == crate::map::TileType::Floor;
        
        if is_above_floor && !side_wall_tiles.is_empty() {
            // This wall is directly above a floor tile, so use a side wall tile
            let index = rng.gen_range(0..side_wall_tiles.len());
            return Some(side_wall_tiles[index]);
        }
        
        // For walls not directly above floor tiles, use top wall tiles
        // But add some variety by occasionally using side tiles for visual interest
        let has_floor_left = x > 0 && map.tiles[y][x-1] == crate::map::TileType::Floor;
        let has_floor_right = x < crate::map::MAP_WIDTH - 1 && map.tiles[y][x+1] == crate::map::TileType::Floor;
        
        // If there are adjacent floor tiles, consider using a side wall for visual interest
        let should_use_side_tile = (has_floor_left || has_floor_right) && rng.gen_bool(0.3); // 30% chance
        
        if should_use_side_tile && !side_wall_tiles.is_empty() {
            let index = rng.gen_range(0..side_wall_tiles.len());
            Some(side_wall_tiles[index])
        } else if !top_wall_tiles.is_empty() {
            // Default to top wall tiles for most cases
            let index = rng.gen_range(0..top_wall_tiles.len());
            Some(top_wall_tiles[index])
        } else if !side_wall_tiles.is_empty() {
            // Fallback to side wall if no top walls are available
            let index = rng.gen_range(0..side_wall_tiles.len());
            Some(side_wall_tiles[index])
        } else {
            // Fallback to any wall tile
            let wall_tiles: Vec<&TileInfo> = biome_tiles.iter()
                .filter(|tile| tile.walkability == TileWalkability::Blocked)
                .collect();
                
            if wall_tiles.is_empty() {
                return None;
            }
            
            let index = rng.gen_range(0..wall_tiles.len());
            Some(wall_tiles[index])
        }
    }
    
    /// Helper method to check if a position has a side wall tile
    /// This is used to determine if a wall should be rendered as a side wall
    fn is_side_wall_at(&self, _biome: BiomeType, x: usize, y: usize, map: &crate::map::TileMap) -> bool {
        // If it's not a wall, it's definitely not a side wall
        if map.tiles[y][x] != crate::map::TileType::Wall {
            return false;
        }
        
        // A wall is a side wall if it's directly above a floor tile
        y > 0 && map.tiles[y-1][x] == crate::map::TileType::Floor
    }
    
    /// Get a varied floor tile for a specific biome and position
    pub fn get_varied_floor_tile(&self, biome: BiomeType, x: usize, y: usize, rng: &mut impl Rng) -> Option<&TileInfo> {
        let biome_tiles = self.biome_tiles.get(&biome)?;
        
        // Filter out any floor tiles that might accidentally be stair tiles
        let floor_tiles: Vec<&TileInfo> = biome_tiles.iter()
            .filter(|tile| 
                tile.walkability == TileWalkability::Walkable && 
                !tile.name.contains("stair") && 
                !tile.name.contains("staircase"))
            .collect();
        
        if floor_tiles.is_empty() {
            return None;
        }
        
        // Use a more complex hash function with prime numbers to reduce visible patterns
        // Add a large prime offset to break up diagonal patterns
        let hash_base = ((x * 7919) + (y * 6971) + (x * y * 2953) + 104729) % floor_tiles.len();
        
        // Increase randomness significantly (50% chance of random tile)
        if rng.gen_bool(0.5) {
            let index = rng.gen_range(0..floor_tiles.len());
            Some(floor_tiles[index])
        } else {
            // For the deterministic case, add more variation by using a secondary hash
            let secondary_hash = ((x * 104729) ^ (y * 15485863) ^ ((x+y) * 32452843)) % floor_tiles.len();
            
            // Choose between primary and secondary hash
            if rng.gen_bool(0.5) {
                Some(floor_tiles[hash_base])
            } else {
                Some(floor_tiles[secondary_hash])
            }
        }
    }
    
    /// Get a door tile for a specific biome
    pub fn get_door_tile(&self, biome: BiomeType) -> Option<&TileInfo> {
        let biome_tiles = self.biome_tiles.get(&biome)?;
        biome_tiles.iter()
            .find(|tile| tile.walkability == TileWalkability::Door)
    }
    
    /// Determine if a position should be part of a path
    /// This uses noise functions to create winding paths
    pub fn is_on_path(&self, x: usize, y: usize) -> bool {
        // Convert coordinates to floating point for smoother calculations
        let fx = x as f32 * 0.15;  // Adjust frequency for wider spacing between paths
        let fy = y as f32 * 0.15;
        
        // Create primary winding path
        let primary_path_value = (fx.sin() * 3.0 + fy.cos() * 3.0).abs();
        let primary_path = primary_path_value < 0.6;  // Thinner primary path
        
        // Create secondary path with different frequency
        let secondary_path_value = ((fx * 0.5).sin() * 4.0 + (fy * 0.5).cos() * 4.0).abs();
        let secondary_path = secondary_path_value < 0.5;  // Even thinner secondary path
        
        // Create path branches/offshoots
        let branch_seed = (x * 7 + y * 13) % 100;
        let branch_path = branch_seed < 15 && (  // Only 15% chance of branch
            ((fx * 0.3).sin() * 2.0 + (fy * 0.3).cos() * 2.0).abs() < 0.4
        );
        
        // Create path width variation (1-2 tiles wide)
        let width_variation = (x * 11 + y * 17) % 10;
        let is_wider_path = width_variation < 4;  // 40% chance of wider path
        
        // Check if position is on any path
        if primary_path || secondary_path || branch_path {
            // For wider paths, include adjacent tiles
            if is_wider_path {
                // Check if this is an edge tile of a path
                let edge_value = if primary_path {
                    primary_path_value
                } else if secondary_path {
                    secondary_path_value
                } else {
                    0.3  // Default for branch paths
                };
                
                // Edge tiles have values close to the threshold
                return edge_value < 0.8;  // Wider threshold for edge tiles
            }
            return true;
        }
        
        false
    }
    
    /// Determine if a position should be part of a path for a specific biome
    /// This creates different path patterns for each biome
    pub fn is_on_biome_path(&self, biome: BiomeType, x: usize, y: usize) -> bool {
        // Convert coordinates to floating point for smoother calculations
        let fx = x as f32 * 0.15;
        let fy = y as f32 * 0.15;
        
        match biome {
            BiomeType::Caves => {
                // Caves biome: More meandering paths with more branches
                let primary_path_value = (fx.sin() * 2.5 + fy.cos() * 2.5).abs();
                let primary_path = primary_path_value < 0.55;
                
                let secondary_path_value = ((fx * 0.4).sin() * 3.5 + (fy * 0.4).cos() * 3.5).abs();
                let secondary_path = secondary_path_value < 0.45;
                
                // More branches in caves biome
                let branch_seed = (x * 9 + y * 11) % 100;
                let branch_path = branch_seed < 20 && (  // 20% chance of branch
                    ((fx * 0.25).sin() * 1.8 + (fy * 0.25).cos() * 1.8).abs() < 0.5
                );
                
                // Width variation - caves paths tend to be wider
                let width_variation = (x * 13 + y * 19) % 10;
                let is_wider_path = width_variation < 6;  // 60% chance of wider path
                
                if primary_path || secondary_path || branch_path {
                    if is_wider_path {
                        let edge_value = if primary_path {
                            primary_path_value
                        } else if secondary_path {
                            secondary_path_value
                        } else {
                            0.35
                        };
                        return edge_value < 0.85;  // Wider threshold for caves paths
                    }
                    return true;
                }
            },
            BiomeType::Groves => {
                // Groves biome: Winding paths with occasional branches
                let primary_path_value = (fx.sin() * 3.0 + fy.cos() * 3.0).abs();
                let primary_path = primary_path_value < 0.6;
                
                let secondary_path_value = ((fx * 0.5).sin() * 4.0 + (fy * 0.5).cos() * 4.0).abs();
                let secondary_path = secondary_path_value < 0.5;
                
                let branch_seed = (x * 7 + y * 13) % 100;
                let branch_path = branch_seed < 15 && (
                    ((fx * 0.3).sin() * 2.0 + (fy * 0.3).cos() * 2.0).abs() < 0.4
                );
                
                // Width variation
                let width_variation = (x * 11 + y * 17) % 10;
                let is_wider_path = width_variation < 4;  // 40% chance of wider path
                
                if primary_path || secondary_path || branch_path {
                    if is_wider_path {
                        let edge_value = if primary_path {
                            primary_path_value
                        } else if secondary_path {
                            secondary_path_value
                        } else {
                            0.3
                        };
                        return edge_value < 0.8;
                    }
                    return true;
                }
            },
            BiomeType::Labyrinth => {
                // Labyrinth biome: Straighter paths with sharp turns
                // Use a different approach for labyrinth - more grid-like paths
                
                // Main horizontal paths
                let h_path_value = (fy * 5.0).sin().abs();
                let h_path = h_path_value < 0.3 && (x * 3 + y * 5) % 7 != 0;  // Occasional gaps
                
                // Main vertical paths
                let v_path_value = (fx * 5.0).sin().abs();
                let v_path = v_path_value < 0.3 && (x * 5 + y * 3) % 7 != 0;  // Occasional gaps
                
                // Diagonal connectors
                let diag_seed = (x * 11 + y * 13) % 100;
                let diag_path = diag_seed < 10 && (  // 10% chance of diagonal connector
                    ((fx + fy) * 0.4).sin().abs() < 0.25
                );
                
                // Width variation - labyrinth paths are mostly narrow
                let width_variation = (x * 7 + y * 23) % 10;
                let is_wider_path = width_variation < 3;  // 30% chance of wider path
                
                if h_path || v_path || diag_path {
                    if is_wider_path {
                        let edge_value = if h_path {
                            h_path_value
                        } else if v_path {
                            v_path_value
                        } else {
                            0.2
                        };
                        return edge_value < 0.6;
                    }
                    return true;
                }
            },
            BiomeType::Catacombs => {
                // Catacombs biome: Straighter paths with sharp turns
                // Use a different approach for catacombs - more grid-like paths
                
                // Main horizontal paths
                let h_path_value = (fy * 5.0).sin().abs();
                let h_path = h_path_value < 0.3 && (x * 3 + y * 5) % 7 != 0;  // Occasional gaps
                
                // Main vertical paths
                let v_path_value = (fx * 5.0).sin().abs();
                let v_path = v_path_value < 0.3 && (x * 5 + y * 3) % 7 != 0;  // Occasional gaps
                
                // Diagonal connectors
                let diag_seed = (x * 11 + y * 13) % 100;
                let diag_path = diag_seed < 10 && (  // 10% chance of diagonal connector
                    ((fx + fy) * 0.4).sin().abs() < 0.25
                );
                
                // Width variation - catacombs paths are mostly narrow
                let width_variation = (x * 7 + y * 23) % 10;
                let is_wider_path = width_variation < 3;  // 30% chance of wider path
                
                if h_path || v_path || diag_path {
                    if is_wider_path {
                        let edge_value = if h_path {
                            h_path_value
                        } else if v_path {
                            v_path_value
                        } else {
                            0.2
                        };
                        return edge_value < 0.6;
                    }
                    return true;
                }
            },
            _ => return self.is_on_path(x, y)  // Use default path logic for other biomes
        }
        
        false
    }
    
    /// Initialize with default tile mappings
    pub fn initialize_default_tiles(&mut self, sprite_assets: &HashMap<String, usize>) {
        // CAVES BIOME
        // Wall tiles for Caves
        if let Some(&index) = sprite_assets.get("dirt wall (top)") {
            self.register_tile("dirt wall (top)", index, TileWalkability::Blocked, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("dirt wall (side)") {
            self.register_tile("dirt wall (side)", index, TileWalkability::Blocked, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("rough stone wall (top)") {
            self.register_tile("rough stone wall (top)", index, TileWalkability::Blocked, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("rough stone wall (side)") {
            self.register_tile("rough stone wall (side)", index, TileWalkability::Blocked, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("igneous wall (top)") {
            self.register_tile("igneous wall (top)", index, TileWalkability::Blocked, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("igneous wall (side)") {
            self.register_tile("igneous wall (side)", index, TileWalkability::Blocked, BiomeType::Caves);
        }
        
        // Floor tiles for Caves
        if let Some(&index) = sprite_assets.get("blank floor (dark grey)") {
            self.register_tile("blank floor (dark grey)", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("floor stone 1") {
            self.register_tile("floor stone 1", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("floor stone 2") {
            self.register_tile("floor stone 2", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("floor stone 3") {
            self.register_tile("floor stone 3", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("blank floor (dark purple)") {
            self.register_tile("blank floor (dark purple)", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("grass 1") {
            self.register_tile("grass 1", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("grass 2") {
            self.register_tile("grass 2", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("grass 3") {
            self.register_tile("grass 3", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("dirt 1") {
            self.register_tile("dirt 1", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("dirt 2") {
            self.register_tile("dirt 2", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("dirt 3") {
            self.register_tile("dirt 3", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        if let Some(&index) = sprite_assets.get("dark brown bg") {
            self.register_tile("dark brown bg", index, TileWalkability::Walkable, BiomeType::Caves);
        }
        
        // GROVES BIOME
        // Wall tiles for Groves
        if let Some(&index) = sprite_assets.get("dirt wall (top)") {
            self.register_tile("dirt wall (top)", index, TileWalkability::Blocked, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("dirt wall (side)") {
            self.register_tile("dirt wall (side)", index, TileWalkability::Blocked, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("rough stone wall (top)") {
            self.register_tile("rough stone wall (top)", index, TileWalkability::Blocked, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("rough stone wall (side)") {
            self.register_tile("rough stone wall (side)", index, TileWalkability::Blocked, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("igneous wall (top)") {
            self.register_tile("igneous wall (top)", index, TileWalkability::Blocked, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("igneous wall (side)") {
            self.register_tile("igneous wall (side)", index, TileWalkability::Blocked, BiomeType::Groves);
        }
        
        // Floor tiles for Groves
        if let Some(&index) = sprite_assets.get("blank green floor") {
            self.register_tile("blank green floor", index, TileWalkability::Walkable, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("dirt 1 (green bg)") {
            self.register_tile("dirt 1 (green bg)", index, TileWalkability::Walkable, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("dirt 2 (green bg)") {
            self.register_tile("dirt 2 (green bg)", index, TileWalkability::Walkable, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("dirt 3 (green bg)") {
            self.register_tile("dirt 3 (green bg)", index, TileWalkability::Walkable, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("grass 1 (green bg)") {
            self.register_tile("grass 1 (green bg)", index, TileWalkability::Walkable, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("grass 2 (green bg)") {
            self.register_tile("grass 2 (green bg)", index, TileWalkability::Walkable, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("grass 3 (green bg)") {
            self.register_tile("grass 3 (green bg)", index, TileWalkability::Walkable, BiomeType::Groves);
        }
        if let Some(&index) = sprite_assets.get("dark brown bg") {
            self.register_tile("dark brown bg", index, TileWalkability::Walkable, BiomeType::Groves);
        }
        
        // LABYRINTH BIOME
        // Wall tiles for Labyrinth
        if let Some(&index) = sprite_assets.get("stone brick wall (top)") {
            self.register_tile("stone brick wall (top)", index, TileWalkability::Blocked, BiomeType::Labyrinth);
        }
        if let Some(&index) = sprite_assets.get("stone brick wall (side 1)") {
            self.register_tile("stone brick wall (side 1)", index, TileWalkability::Blocked, BiomeType::Labyrinth);
        }
        if let Some(&index) = sprite_assets.get("stone brick wall (side 2)") {
            self.register_tile("stone brick wall (side 2)", index, TileWalkability::Blocked, BiomeType::Labyrinth);
        }
        if let Some(&index) = sprite_assets.get("large stone wall (top)") {
            self.register_tile("large stone wall (top)", index, TileWalkability::Blocked, BiomeType::Labyrinth);
        }
        if let Some(&index) = sprite_assets.get("large stone wall (side)") {
            self.register_tile("large stone wall (side)", index, TileWalkability::Blocked, BiomeType::Labyrinth);
        }
        
        // Floor tiles for Labyrinth
        if let Some(&index) = sprite_assets.get("blank red floor") {
            self.register_tile("blank red floor", index, TileWalkability::Walkable, BiomeType::Labyrinth);
        }
        if let Some(&index) = sprite_assets.get("red stone floor 1 (red bg)") {
            self.register_tile("red stone floor 1 (red bg)", index, TileWalkability::Walkable, BiomeType::Labyrinth);
        }
        if let Some(&index) = sprite_assets.get("red stone floor 2 (red bg)") {
            self.register_tile("red stone floor 2 (red bg)", index, TileWalkability::Walkable, BiomeType::Labyrinth);
        }
        if let Some(&index) = sprite_assets.get("red stone floor 3 (red bg)") {
            self.register_tile("red stone floor 3 (red bg)", index, TileWalkability::Walkable, BiomeType::Labyrinth);
        }
        if let Some(&index) = sprite_assets.get("dark brown bg") {
            self.register_tile("dark brown bg", index, TileWalkability::Walkable, BiomeType::Labyrinth);
        }
        if let Some(&index) = sprite_assets.get("bones 1 (dark brown bg)") {
            self.register_tile("bones 1 (dark brown bg)", index, TileWalkability::Walkable, BiomeType::Labyrinth);
        }
        if let Some(&index) = sprite_assets.get("bones 2 (dark brown bg)") {
            self.register_tile("bones 2 (dark brown bg)", index, TileWalkability::Walkable, BiomeType::Labyrinth);
        }
        if let Some(&index) = sprite_assets.get("bones 3 (dark brown bg)") {
            self.register_tile("bones 3 (dark brown bg)", index, TileWalkability::Walkable, BiomeType::Labyrinth);
        }
        
        // CATACOMBS BIOME
        // Wall tiles for Catacombs
        if let Some(&index) = sprite_assets.get("stone brick wall (top)") {
            self.register_tile("stone brick wall (top)", index, TileWalkability::Blocked, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("stone brick wall (side 1)") {
            self.register_tile("stone brick wall (side 1)", index, TileWalkability::Blocked, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("stone brick wall (side 2)") {
            self.register_tile("stone brick wall (side 2)", index, TileWalkability::Blocked, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("catacombs / skull wall (top)") {
            self.register_tile("catacombs / skull wall (top)", index, TileWalkability::Blocked, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("catacombs / skull walls (side)") {
            self.register_tile("catacombs / skull walls (side)", index, TileWalkability::Blocked, BiomeType::Catacombs);
        }
        
        // Floor tiles for Catacombs
        if let Some(&index) = sprite_assets.get("bone 1") {
            self.register_tile("bone 1", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("bone 2") {
            self.register_tile("bone 2", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("bone 3") {
            self.register_tile("bone 3", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("dark brown bg") {
            self.register_tile("dark brown bg", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("bones 1 (dark brown bg)") {
            self.register_tile("bones 1 (dark brown bg)", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("bones 2 (dark brown bg)") {
            self.register_tile("bones 2 (dark brown bg)", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("bones 3 (dark brown bg)") {
            self.register_tile("bones 3 (dark brown bg)", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("blank floor (dark grey)") {
            self.register_tile("blank floor (dark grey)", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        
        // Add red floor tiles to Catacombs biome
        if let Some(&index) = sprite_assets.get("blank red floor") {
            self.register_tile("blank red floor", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("red stone floor 1 (red bg)") {
            self.register_tile("red stone floor 1 (red bg)", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("red stone floor 2 (red bg)") {
            self.register_tile("red stone floor 2 (red bg)", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("red stone floor 3 (red bg)") {
            self.register_tile("red stone floor 3 (red bg)", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        
        // Door tiles for all biomes
        if let Some(&index) = sprite_assets.get("framed door 1 (shut)") {
            self.register_tile("framed door 1 (shut)", index, TileWalkability::Door, BiomeType::Caves);
            self.register_tile("framed door 1 (shut)", index, TileWalkability::Door, BiomeType::Groves);
            self.register_tile("framed door 1 (shut)", index, TileWalkability::Door, BiomeType::Labyrinth);
            self.register_tile("framed door 1 (shut)", index, TileWalkability::Door, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("door 1") {
            self.register_tile("door 1", index, TileWalkability::Door, BiomeType::Caves);
            self.register_tile("door 1", index, TileWalkability::Door, BiomeType::Groves);
            self.register_tile("door 1", index, TileWalkability::Door, BiomeType::Labyrinth);
            self.register_tile("door 1", index, TileWalkability::Door, BiomeType::Catacombs);
        }
        
        // Stair tiles for all biomes
        if let Some(&index) = sprite_assets.get("staircase down").or_else(|| sprite_assets.get("stairs down")) {
            self.register_tile("stairs down", index, TileWalkability::Walkable, BiomeType::Caves);
            self.register_tile("stairs down", index, TileWalkability::Walkable, BiomeType::Groves);
            self.register_tile("stairs down", index, TileWalkability::Walkable, BiomeType::Labyrinth);
            self.register_tile("stairs down", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("staircase up").or_else(|| sprite_assets.get("stairs up")) {
            self.register_tile("stairs up", index, TileWalkability::Walkable, BiomeType::Caves);
            self.register_tile("stairs up", index, TileWalkability::Walkable, BiomeType::Groves);
            self.register_tile("stairs up", index, TileWalkability::Walkable, BiomeType::Labyrinth);
            self.register_tile("stairs up", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
    }

    /// Get a stairs down tile for a specific biome
    pub fn get_stairs_down_tile(&self, biome: BiomeType) -> Option<&TileInfo> {
        self.biome_tiles.get(&biome)?
            .iter()
            .find(|tile| tile.name.contains("stairs down") || tile.name.contains("staircase down"))
    }

    /// Get a stairs up tile for a specific biome
    pub fn get_stairs_up_tile(&self, biome: BiomeType) -> Option<&TileInfo> {
        self.biome_tiles.get(&biome)?
            .iter()
            .find(|tile| tile.name.contains("stairs up") || tile.name.contains("staircase up"))
    }
} 