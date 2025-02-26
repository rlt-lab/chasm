use bevy::prelude::*;
use bevy::sprite::TextureAtlasSprite;
use crate::map::{TileMap, TileType, MAP_WIDTH, MAP_HEIGHT};
use crate::assets::{SpriteAssets, TextureAtlases};
use crate::visibility::{VisibilityMap, TileVisibility};
use crate::biome::{BiomeManager, TileWalkability};
use crate::input::TILE_SIZE;

#[derive(Component)]
pub struct TilePos {
    pub x: usize,
    pub y: usize,
}

// Grid line component
#[derive(Component)]
pub struct GridLine;

// Resource to track tile entities
#[derive(Resource, Default)]
pub struct TileEntities {
    pub entities: Vec<Entity>,
}

pub fn spawn_tiles(
    commands: &mut Commands,
    map: &TileMap,
    texture_atlases: &Res<TextureAtlases>,
    sprite_assets: &Res<SpriteAssets>,
    biome_manager: Option<&Res<BiomeManager>>,
) {
    let mut rng = rand::thread_rng();
    
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let (x_pos, y_pos) = (
                x as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                y as f32 * TILE_SIZE + (TILE_SIZE / 2.0)
            );

            // Get the biome for this tile
            let biome = map.get_biome_at(x, y);
            
            // Convert TileType to TileWalkability for rendering
            let walkability = match map.tiles[y][x] {
                TileType::Floor => TileWalkability::Walkable,
                TileType::Wall => TileWalkability::Blocked,
                TileType::Door => TileWalkability::Door,
                TileType::SecretDoor => TileWalkability::Door,
                TileType::StairsDown => TileWalkability::Walkable,
                TileType::StairsUp => TileWalkability::Walkable,
            };
            
            // Determine sprite index based on tile type and biome
            let (sprite_index, z_pos) = if let Some(biome_mgr) = biome_manager {
                match map.tiles[y][x] {
                    TileType::Wall => {
                        if let Some(tile_info) = biome_mgr.get_wall_tile_for_position(biome, x, y, map, &mut rng) {
                            (tile_info.sprite_index, 1.0)
                        } else {
                            (crate::assets::get_random_wall_tile(sprite_assets), 1.0)
                        }
                    }
                    TileType::Floor => {
                        if let Some(tile_info) = biome_mgr.get_varied_floor_tile(biome, x, y, &mut rng) {
                            (tile_info.sprite_index, 0.0)
                        } else {
                            (crate::assets::get_random_floor_tile(sprite_assets), 0.0)
                        }
                    }
                    TileType::Door => {
                        if let Some(tile_info) = biome_mgr.get_door_tile(biome) {
                            (tile_info.sprite_index, 1.0)
                        } else {
                            (crate::assets::get_door_sprite(sprite_assets), 1.0)
                        }
                    }
                    TileType::SecretDoor => {
                        if let Some(tile_info) = biome_mgr.get_wall_tile_for_position(biome, x, y, map, &mut rng) {
                            (tile_info.sprite_index, 1.0)
                        } else {
                            (crate::assets::get_random_wall_tile(sprite_assets), 1.0)
                        }
                    }
                    TileType::StairsDown => {
                        if let Some(tile_info) = biome_mgr.get_stairs_down_tile(biome) {
                            (tile_info.sprite_index, 0.0)
                        } else {
                            (crate::assets::get_stairs_down_sprite(sprite_assets), 0.0)
                        }
                    }
                    TileType::StairsUp => {
                        if let Some(tile_info) = biome_mgr.get_stairs_up_tile(biome) {
                            (tile_info.sprite_index, 0.0)
                        } else {
                            (crate::assets::get_stairs_up_sprite(sprite_assets), 0.0)
                        }
                    }
                }
            } else {
                match map.tiles[y][x] {
                    TileType::Wall => (crate::assets::get_random_wall_tile(sprite_assets), 1.0),
                    TileType::Floor => (crate::assets::get_random_floor_tile(sprite_assets), 0.0),
                    TileType::Door => (crate::assets::get_door_sprite(sprite_assets), 1.0),
                    TileType::SecretDoor => (crate::assets::get_random_wall_tile(sprite_assets), 1.0),
                    TileType::StairsDown => (crate::assets::get_stairs_down_sprite(sprite_assets), 0.0),
                    TileType::StairsUp => (crate::assets::get_stairs_up_sprite(sprite_assets), 0.0),
                }
            };

            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: texture_atlases.tiles.clone(),
                    sprite: TextureAtlasSprite {
                        index: sprite_index,
                        color: Color::rgba(1.0, 1.0, 1.0, 1.0), // Start fully visible
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(x_pos, y_pos, z_pos)),
                    ..default()
                },
                TilePos { x, y },
                TileVisibility {
                    visible: true,
                    previously_seen: true,
                },
                crate::components::Tile {
                    tile_type: map.tiles[y][x],
                    walkability,
                    biome,
                },
            ));
        }
    }
}

pub fn update_tile_visibility(
    visibility_map: Res<VisibilityMap>,
    mut query: Query<(&TilePos, &mut TextureAtlasSprite, &mut TileVisibility)>,
) {
    for (pos, mut sprite, mut tile_vis) in query.iter_mut() {
        if visibility_map.visible_tiles[pos.y][pos.x] {
            sprite.color.set_a(1.0);
            tile_vis.previously_seen = true;
            tile_vis.visible = true;
        } else if visibility_map.previously_seen[pos.y][pos.x] {
            sprite.color.set_a(0.3); // Dimmer for previously seen tiles
            tile_vis.previously_seen = true;
            tile_vis.visible = false;
        } else {
            sprite.color.set_a(0.0); // Completely invisible
            tile_vis.previously_seen = false;
            tile_vis.visible = false;
        }
    }
}

// Function to spawn grid lines
pub fn spawn_grid_lines(_commands: &mut Commands) {
    // Grid lines are currently disabled
}

// Function to toggle grid visibility
pub fn toggle_grid_visibility(
    _grid_query: Query<&mut Visibility, With<GridLine>>,
    _keyboard_input: Res<Input<KeyCode>>,
) {
    // Grid visibility toggle is currently disabled
}

pub fn generate_map_visuals(
    commands: &mut Commands,
    map: &TileMap,
    _asset_server: &Res<AssetServer>,
    sprite_assets: &Res<SpriteAssets>,
    texture_atlases: &Res<TextureAtlases>,
    biome_manager: &Res<BiomeManager>,
    tile_entities: &mut TileEntities,
) {
    // Clear existing tile entities
    tile_entities.entities.clear();
    
    // Spawn new tiles
    spawn_tiles(commands, map, texture_atlases, sprite_assets, Some(biome_manager));
    
    // Spawn grid lines
    spawn_grid_lines(commands);
}
