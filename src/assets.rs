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
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut sprite_map = HashMap::new();

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
                    let index = ((row - 1) * 16) + col;
                    
                    // Only add if the index is within bounds (16x16 grid = 256 sprites)
                    if index < 256 {
                        sprite_map.insert(name, index);
                    }
                }
            }
        }
    }

    // If the map is empty, add some fallback sprites
    if sprite_map.is_empty() {
        // Add some fallback sprites
        sprite_map.insert("wall".to_string(), 48);  // Default wall sprite
        sprite_map.insert("floor".to_string(), 96); // Default floor sprite
        sprite_map.insert("door".to_string(), 68);  // Default door sprite
    }

    Ok(sprite_map)
}

/// Load all sprite assets
pub fn load_sprite_assets(
    mut commands: &mut Commands,
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
        16, 16,
        None, None
    );
    let tiles_atlas_handle = texture_atlases.add(tiles_atlas);
    
    let characters_handle = asset_server.load("sprites/rogues.png");
    let characters_atlas = TextureAtlas::from_grid(
        characters_handle,
        Vec2::new(32.0, 32.0),
        16, 16,
        None, None
    );
    let characters_atlas_handle = texture_atlases.add(characters_atlas);
    
    let monsters_handle = asset_server.load("sprites/monsters.png");
    let monsters_atlas = TextureAtlas::from_grid(
        monsters_handle,
        Vec2::new(32.0, 32.0),
        16, 16,
        None, None
    );
    let monsters_atlas_handle = texture_atlases.add(monsters_atlas);
    
    let items_handle = asset_server.load("sprites/items.png");
    let items_atlas = TextureAtlas::from_grid(
        items_handle,
        Vec2::new(32.0, 32.0),
        16, 16,
        None, None
    );
    let items_atlas_handle = texture_atlases.add(items_atlas);
    
    let animals_handle = asset_server.load("sprites/animals.png");
    let animals_atlas = TextureAtlas::from_grid(
        animals_handle,
        Vec2::new(32.0, 32.0),
        16, 16,
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
    use rand::seq::SliceRandom;
    
    // Collect all floor tile sprites
    let floor_sprites: Vec<&usize> = sprite_assets.tile_sprites.iter()
        .filter(|(name, _)| name.contains("floor") || name.contains("stone") || name.contains("grass"))
        .map(|(_, index)| index)
        .collect();
    
    // Choose a random floor tile or use fallback
    if floor_sprites.is_empty() {
        // Fallback to a default floor tile index
        96 // Default floor tile index
    } else {
        **floor_sprites.choose(&mut rand::thread_rng()).unwrap_or(&&96)
    }
}

/// Get a random wall tile sprite index
pub fn get_random_wall_tile(sprite_assets: &SpriteAssets) -> usize {
    use rand::seq::SliceRandom;
    
    // Collect all wall tile sprites
    let wall_sprites: Vec<&usize> = sprite_assets.tile_sprites.iter()
        .filter(|(name, _)| name.contains("wall"))
        .map(|(_, index)| index)
        .collect();
    
    // Choose a random wall tile or use fallback
    if wall_sprites.is_empty() {
        // Fallback to a default wall tile index
        48 // Default wall tile index
    } else {
        **wall_sprites.choose(&mut rand::thread_rng()).unwrap_or(&&48)
    }
}

/// Get a door tile sprite index
pub fn get_door_sprite(sprite_assets: &SpriteAssets) -> usize {
    *sprite_assets.tile_sprites.get("framed door 1 (shut)")
        .or_else(|| sprite_assets.tile_sprites.get("door"))
        .unwrap_or(&68) // Default door tile index
} 