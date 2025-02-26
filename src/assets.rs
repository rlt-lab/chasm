use bevy::prelude::*;
use bevy::sprite::TextureAtlas;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

/// Resource that holds all sprite mappings
#[derive(Resource)]
pub struct SpriteAssets {
    pub tile_sprites: HashMap<String, usize>,
    pub character_sprites: HashMap<String, usize>,
    pub monster_sprites: HashMap<String, usize>,
    pub item_sprites: HashMap<String, usize>,
    pub animal_sprites: HashMap<String, usize>,
}

/// Resource that holds all texture atlas handles
#[derive(Resource)]
pub struct TextureAtlases {
    pub tiles: Handle<TextureAtlas>,
    pub characters: Handle<TextureAtlas>,
    pub monsters: Handle<TextureAtlas>,
    pub items: Handle<TextureAtlas>,
    pub animals: Handle<TextureAtlas>,
}

impl Default for SpriteAssets {
    fn default() -> Self {
        Self {
            tile_sprites: HashMap::new(),
            character_sprites: HashMap::new(),
            monster_sprites: HashMap::new(),
            item_sprites: HashMap::new(),
            animal_sprites: HashMap::new(),
        }
    }
}

/// Parse a sprite sheet metadata file and return a mapping of sprite names to indices
fn parse_sprite_metadata(file_path: &str) -> io::Result<HashMap<String, usize>> {
    let path = Path::new("assets").join(file_path);
    
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            // Create an empty map with fallbacks
            let mut empty_map = HashMap::new();
            // Add fallback sprites
            empty_map.insert("wall".to_string(), 21);
            empty_map.insert("floor".to_string(), 126);
            empty_map.insert("door".to_string(), 338);
            empty_map.insert("staircase down".to_string(), 343);
            empty_map.insert("staircase up".to_string(), 344);
            return Ok(empty_map);
        }
    };
    
    let reader = io::BufReader::new(file);
    let mut sprite_map = HashMap::new();

    // Determine columns per row based on the file
    let columns_per_row = match file_path {
        "sprites/tiles.txt" => 21,    // 672/32 = 21
        "sprites/rogues.txt" => 6,    // 192/32 = 6
        "sprites/monsters.txt" => 12, // 384/32 = 12
        "sprites/items.txt" => 8,     // 256/32 = 8
        "sprites/animals.txt" => 9,   // 288/32 = 9
        _ => 16, // Default fallback
    };

    // Determine max index based on the file
    let max_index = match file_path {
        "sprites/tiles.txt" => 21 * 24,    // 21 columns × 24 rows = 504
        "sprites/rogues.txt" => 6 * 7,     // 6 columns × 7 rows = 42
        "sprites/monsters.txt" => 12 * 13, // 12 columns × 13 rows = 156
        "sprites/items.txt" => 8 * 22,     // 8 columns × 22 rows = 176
        "sprites/animals.txt" => 9 * 16,   // 9 columns × 16 rows = 144
        _ => 256, // Default fallback
    };

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Parse lines like "1.a. dirt wall (top)"
        if let Some(pos) = line.find('.') {
            if let Some(second_pos) = line[pos+1..].find('.') {
                let row_str = line[..pos].trim();
                if let Ok(row) = row_str.parse::<usize>() {
                    let col_char = line[pos+1..pos+1+second_pos].trim();
                    let name = line[pos+1+second_pos+1..].trim().to_string();
                    
                    // Convert column letter to index (a=0, b=1, etc.)
                    let col = match col_char {
                        "a" => 0,
                        "b" => 1,
                        "c" => 2,
                        "d" => 3,
                        "e" => 4,
                        "f" => 5,
                        "g" => 6,
                        "h" => 7,
                        "i" => 8,
                        "j" => 9,
                        "k" => 10,
                        "l" => 11,
                        "m" => 12,
                        "n" => 13,
                        "o" => 14,
                        "p" => 15,
                        _ => 0,
                    };
                    
                    // Calculate index in the sprite sheet (row * columns_per_row + column)
                    // Ensure row is 0-indexed for calculation
                    let index = ((row - 1) * columns_per_row) + col;
                    
                    // Only add if the index is within bounds
                    if index < max_index {
                        sprite_map.insert(name, index);
                    }
                }
            }
        }
    }

    // If the map is empty, add some fallback sprites
    if sprite_map.is_empty() {
        // Add some fallback sprites with correct indices
        // Wall: 2.a rough stone wall (top) = ((2-1) * 21) + 0 = 21
        sprite_map.insert("wall".to_string(), 21);
        
        // Floor: 7.a blank floor (dark grey) = ((7-1) * 21) + 0 = 126
        sprite_map.insert("floor".to_string(), 126);
        
        // Door: 17.c framed door 1 (shut) = ((17-1) * 21) + 2 = 338
        sprite_map.insert("door".to_string(), 338);
        
        // Stairs
        sprite_map.insert("staircase down".to_string(), 343); // 17.h
        sprite_map.insert("staircase up".to_string(), 344);   // 17.i
    } else {
        // Check if we have floor tiles
        let has_floor = sprite_map.contains_key("blank floor (dark grey)") || 
                        sprite_map.contains_key("floor") ||
                        sprite_map.contains_key("floor stone 1");
                        
        if !has_floor {
            sprite_map.insert("floor".to_string(), 126); // 7.a blank floor
        }
    }

    Ok(sprite_map)
}

/// Load all sprite assets
pub fn load_sprite_assets(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) -> io::Result<()> {
    // Load sprite metadata
    let tile_sprites = parse_sprite_metadata("sprites/tiles.txt")?;
    let character_sprites = parse_sprite_metadata("sprites/rogues.txt")?;
    let monster_sprites = parse_sprite_metadata("sprites/monsters.txt")?;
    let item_sprites = parse_sprite_metadata("sprites/items.txt")?;
    let animal_sprites = parse_sprite_metadata("sprites/animals.txt")?;
    
    // Create sprite assets resource
    commands.insert_resource(SpriteAssets {
        tile_sprites,
        character_sprites,
        monster_sprites,
        item_sprites,
        animal_sprites,
    });
    
    // Load texture atlases
    let tiles_handle = asset_server.load("sprites/tiles.png");
    let tiles_atlas = TextureAtlas::from_grid(
        tiles_handle,
        Vec2::new(32.0, 32.0),
        21, 24,  // 672/32 = 21, 768/32 = 24
        None, None
    );
    let tiles_atlas_handle = texture_atlases.add(tiles_atlas);
    
    let characters_handle = asset_server.load("sprites/rogues.png");
    let characters_atlas = TextureAtlas::from_grid(
        characters_handle,
        Vec2::new(32.0, 32.0),
        6, 7,  // 192/32 = 6, 224/32 = 7
        None, None
    );
    let characters_atlas_handle = texture_atlases.add(characters_atlas);
    
    let monsters_handle = asset_server.load("sprites/monsters.png");
    let monsters_atlas = TextureAtlas::from_grid(
        monsters_handle,
        Vec2::new(32.0, 32.0),
        12, 13,  // 384/32 = 12, 416/32 = 13
        None, None
    );
    let monsters_atlas_handle = texture_atlases.add(monsters_atlas);
    
    let items_handle = asset_server.load("sprites/items.png");
    let items_atlas = TextureAtlas::from_grid(
        items_handle,
        Vec2::new(32.0, 32.0),
        8, 22,  // 256/32 = 8, 704/32 = 22
        None, None
    );
    let items_atlas_handle = texture_atlases.add(items_atlas);
    
    let animals_handle = asset_server.load("sprites/animals.png");
    let animals_atlas = TextureAtlas::from_grid(
        animals_handle,
        Vec2::new(32.0, 32.0),
        9, 16,  // 288/32 = 9, 512/32 = 16
        None, None
    );
    let animals_atlas_handle = texture_atlases.add(animals_atlas);
    
    // Create texture atlases resource
    commands.insert_resource(TextureAtlases {
        tiles: tiles_atlas_handle,
        characters: characters_atlas_handle,
        monsters: monsters_atlas_handle,
        items: items_atlas_handle,
        animals: animals_atlas_handle,
    });
    
    Ok(())
}

/// Helper functions to get sprite indices by name
pub fn get_tile_sprite(sprite_assets: &SpriteAssets, name: &str) -> usize {
    *sprite_assets.tile_sprites.get(name).unwrap_or(&0)
}

pub fn get_character_sprite(sprite_assets: &SpriteAssets, name: &str) -> usize {
    *sprite_assets.character_sprites.get(name).unwrap_or(&0)
}

pub fn get_monster_sprite(sprite_assets: &SpriteAssets, name: &str) -> usize {
    *sprite_assets.monster_sprites.get(name).unwrap_or(&0)
}

pub fn get_item_sprite(sprite_assets: &SpriteAssets, name: &str) -> usize {
    *sprite_assets.item_sprites.get(name).unwrap_or(&0)
}

pub fn get_animal_sprite(sprite_assets: &SpriteAssets, name: &str) -> usize {
    *sprite_assets.animal_sprites.get(name).unwrap_or(&0)
}

/// Get a random floor tile sprite index
pub fn get_random_floor_tile(sprite_assets: &SpriteAssets) -> usize {
    println!("DEBUG: Available tile sprites: {:?}", sprite_assets.tile_sprites.keys().collect::<Vec<_>>());
    
    // Try to get a specific floor tile
    let floor_index_opt = sprite_assets.tile_sprites.get("blank floor (dark grey)")
        .or_else(|| sprite_assets.tile_sprites.get("blank floor (dark purple)"))
        .or_else(|| sprite_assets.tile_sprites.get("blank green floor"))
        .or_else(|| sprite_assets.tile_sprites.get("floor stone 1"))
        .or_else(|| sprite_assets.tile_sprites.get("floor stone 2"))
        .or_else(|| sprite_assets.tile_sprites.get("floor stone 3"))
        .or_else(|| sprite_assets.tile_sprites.get("floor"))
        .or_else(|| sprite_assets.tile_sprites.get("stone floor 1"));
    
    println!("DEBUG: Floor index option: {:?}", floor_index_opt);
    
    // Get the floor sprite index, ensuring it's not a stair sprite
    let floor_index = if let Some(&index) = floor_index_opt {
        index
    } else {
        // If no floor tile is found, use a safe index that corresponds to a floor tile
        // 7.a is blank floor (dark grey) at index ((7-1) * 21) + 0 = 126
        println!("DEBUG: No floor tile found, using fallback index 126");
        126
    };
    
    println!("DEBUG: Selected floor index: {}", floor_index);
    floor_index
}

/// Get a random wall tile sprite index
pub fn get_random_wall_tile(sprite_assets: &SpriteAssets) -> usize {
    *sprite_assets.tile_sprites.get("wall").unwrap_or(&0)
}

/// Get a door sprite index
pub fn get_door_sprite(sprite_assets: &SpriteAssets) -> usize {
    *sprite_assets.tile_sprites.get("door").unwrap_or(&0)
}

/// Get stairs down sprite index
pub fn get_stairs_down_sprite(sprite_assets: &SpriteAssets) -> usize {
    // Try to get from sprite map first, fallback to a safe index
    // For 17.h in a 21-column grid: ((17-1) * 21) + 7 = 343
    *sprite_assets.tile_sprites.get("staircase down")
        .or_else(|| sprite_assets.tile_sprites.get("stairs down"))
        .unwrap_or(&343)
}

/// Get stairs up sprite index
pub fn get_stairs_up_sprite(sprite_assets: &SpriteAssets) -> usize {
    // Try to get from sprite map first, fallback to a safe index
    // For 17.i in a 21-column grid: ((17-1) * 21) + 8 = 344
    *sprite_assets.tile_sprites.get("staircase up")
        .or_else(|| sprite_assets.tile_sprites.get("stairs up"))
        .unwrap_or(&344)
} 