use bevy::prelude::*;
use std::collections::HashMap;
use rand::Rng;

/// Represents different biome types in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeType {
    Stone,      // Standard dungeon with stone walls and floors
    Dirt,       // Earthy areas with dirt walls and floors
    Catacombs,  // Areas with skull walls and bone floors
    Grass,      // Overgrown areas with grass and plants
    Igneous,    // Volcanic/heated areas with igneous rock
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
        let tile_info = TileInfo {
            name: name.to_string(),
            sprite_index,
            walkability,
            biome,
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
    
    /// Get a varied floor tile for a specific biome
    /// This provides more visual variety than the basic random selection
    pub fn get_varied_floor_tile(&self, biome: BiomeType, x: usize, y: usize, rng: &mut impl Rng) -> Option<&TileInfo> {
        let biome_tiles = self.biome_tiles.get(&biome)?;
        let walkable_tiles: Vec<&TileInfo> = biome_tiles.iter()
            .filter(|tile| tile.walkability == TileWalkability::Walkable)
            .collect();
            
        if walkable_tiles.is_empty() {
            return None;
        }
        
        // Use position and a hash function to create deterministic but varied patterns
        let position_hash = ((x * 7 + y * 13) % 100) as f32 / 100.0;
        
        // Different selection strategies based on position hash
        if position_hash < 0.6 {
            // 60% chance - completely random selection
            let index = rng.gen_range(0..walkable_tiles.len());
            Some(walkable_tiles[index])
        } else if position_hash < 0.85 {
            // 25% chance - select based on position parity (creates checkerboard-like patterns)
            let pattern_index = (x + y) % walkable_tiles.len();
            Some(walkable_tiles[pattern_index])
        } else {
            // 15% chance - select based on another pattern
            let pattern_index = (x * x + y * y) % walkable_tiles.len();
            Some(walkable_tiles[pattern_index])
        }
    }
    
    /// Get a door tile for a specific biome
    pub fn get_door_tile(&self, biome: BiomeType) -> Option<&TileInfo> {
        let biome_tiles = self.biome_tiles.get(&biome)?;
        biome_tiles.iter()
            .find(|tile| tile.walkability == TileWalkability::Door)
    }
    
    /// Initialize with default tile mappings
    pub fn initialize_default_tiles(&mut self, sprite_assets: &HashMap<String, usize>) {
        // Stone biome tiles
        if let Some(&index) = sprite_assets.get("rough stone wall (top)") {
            self.register_tile("rough stone wall (top)", index, TileWalkability::Blocked, BiomeType::Stone);
        }
        if let Some(&index) = sprite_assets.get("rough stone wall (side)") {
            self.register_tile("rough stone wall (side)", index, TileWalkability::Blocked, BiomeType::Stone);
        }
        if let Some(&index) = sprite_assets.get("stone brick wall (top)") {
            self.register_tile("stone brick wall (top)", index, TileWalkability::Blocked, BiomeType::Stone);
        }
        if let Some(&index) = sprite_assets.get("stone floor 1") {
            self.register_tile("stone floor 1", index, TileWalkability::Walkable, BiomeType::Stone);
        }
        if let Some(&index) = sprite_assets.get("stone floor 2") {
            self.register_tile("stone floor 2", index, TileWalkability::Walkable, BiomeType::Stone);
        }
        if let Some(&index) = sprite_assets.get("stone floor 3") {
            self.register_tile("stone floor 3", index, TileWalkability::Walkable, BiomeType::Stone);
        }
        
        // Dirt biome tiles
        if let Some(&index) = sprite_assets.get("dirt wall (top)") {
            self.register_tile("dirt wall (top)", index, TileWalkability::Blocked, BiomeType::Dirt);
        }
        if let Some(&index) = sprite_assets.get("dirt wall (side)") {
            self.register_tile("dirt wall (side)", index, TileWalkability::Blocked, BiomeType::Dirt);
        }
        if let Some(&index) = sprite_assets.get("dirt 1") {
            self.register_tile("dirt 1", index, TileWalkability::Walkable, BiomeType::Dirt);
        }
        if let Some(&index) = sprite_assets.get("dirt 2") {
            self.register_tile("dirt 2", index, TileWalkability::Walkable, BiomeType::Dirt);
        }
        if let Some(&index) = sprite_assets.get("dirt 3") {
            self.register_tile("dirt 3", index, TileWalkability::Walkable, BiomeType::Dirt);
        }
        
        // Catacombs biome tiles
        if let Some(&index) = sprite_assets.get("catacombs / skull wall (top)") {
            self.register_tile("catacombs / skull wall (top)", index, TileWalkability::Blocked, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("catacombs / skull walls (side)") {
            self.register_tile("catacombs / skull walls (side)", index, TileWalkability::Blocked, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("bone 1") {
            self.register_tile("bone 1", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("bone 2") {
            self.register_tile("bone 2", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        if let Some(&index) = sprite_assets.get("bone 3") {
            self.register_tile("bone 3", index, TileWalkability::Walkable, BiomeType::Catacombs);
        }
        
        // Grass biome tiles
        if let Some(&index) = sprite_assets.get("large stone wall (top)") {
            self.register_tile("large stone wall (top)", index, TileWalkability::Blocked, BiomeType::Grass);
        }
        if let Some(&index) = sprite_assets.get("large stone wall (side)") {
            self.register_tile("large stone wall (side)", index, TileWalkability::Blocked, BiomeType::Grass);
        }
        if let Some(&index) = sprite_assets.get("grass 1") {
            self.register_tile("grass 1", index, TileWalkability::Walkable, BiomeType::Grass);
        }
        if let Some(&index) = sprite_assets.get("grass 2") {
            self.register_tile("grass 2", index, TileWalkability::Walkable, BiomeType::Grass);
        }
        if let Some(&index) = sprite_assets.get("grass 3") {
            self.register_tile("grass 3", index, TileWalkability::Walkable, BiomeType::Grass);
        }
        
        // Igneous biome tiles
        if let Some(&index) = sprite_assets.get("igneous wall (top)") {
            self.register_tile("igneous wall (top)", index, TileWalkability::Blocked, BiomeType::Igneous);
        }
        if let Some(&index) = sprite_assets.get("igneous wall (side)") {
            self.register_tile("igneous wall (side)", index, TileWalkability::Blocked, BiomeType::Igneous);
        }
        if let Some(&index) = sprite_assets.get("red stone floor 1 (red bg)") {
            self.register_tile("red stone floor 1", index, TileWalkability::Walkable, BiomeType::Igneous);
        }
        if let Some(&index) = sprite_assets.get("red stone floor 2 (red bg)") {
            self.register_tile("red stone floor 2", index, TileWalkability::Walkable, BiomeType::Igneous);
        }
        if let Some(&index) = sprite_assets.get("red stone floor 3 (red bg)") {
            self.register_tile("red stone floor 3", index, TileWalkability::Walkable, BiomeType::Igneous);
        }
        
        // Door tiles for all biomes
        if let Some(&index) = sprite_assets.get("framed door 1 (shut)") {
            self.register_tile("framed door 1 (shut)", index, TileWalkability::Door, BiomeType::Stone);
            self.register_tile("framed door 1 (shut)", index, TileWalkability::Door, BiomeType::Dirt);
            self.register_tile("framed door 1 (shut)", index, TileWalkability::Door, BiomeType::Catacombs);
            self.register_tile("framed door 1 (shut)", index, TileWalkability::Door, BiomeType::Grass);
            self.register_tile("framed door 1 (shut)", index, TileWalkability::Door, BiomeType::Igneous);
        }
        if let Some(&index) = sprite_assets.get("door 1") {
            self.register_tile("door 1", index, TileWalkability::Door, BiomeType::Stone);
            self.register_tile("door 1", index, TileWalkability::Door, BiomeType::Dirt);
            self.register_tile("door 1", index, TileWalkability::Door, BiomeType::Catacombs);
            self.register_tile("door 1", index, TileWalkability::Door, BiomeType::Grass);
            self.register_tile("door 1", index, TileWalkability::Door, BiomeType::Igneous);
        }
    }
} 