use bevy::prelude::*;
use crate::map::{TileMap, TileType, MAP_WIDTH, MAP_HEIGHT};
use crate::GameAssets;
use rand::Rng;
const TILE_SIZE: f32 = 32.0;

pub fn spawn_tiles(
    commands: &mut Commands,
    map: &TileMap,
    game_assets: &Res<GameAssets>,
) {
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let (x_pos, y_pos) = (
                x as f32 * TILE_SIZE + (TILE_SIZE / 2.0),
                y as f32 * TILE_SIZE + (TILE_SIZE / 2.0)
            );

            let (sprite_index, z_pos) = match map.tiles[y][x] {
                TileType::Wall => {
                    // Use sprite 4.a from the sprite sheet (index 48)
                    (48, 1.0)  // Higher z_pos to ensure walls render on top
                }
                TileType::Floor => {
                    let mut rng = rand::thread_rng();
                    let row = rng.gen_range(6..9); // Rows 6,7,8 correspond to 7,8,9
                    let col = rng.gen_range(0..7); // Columns a-g (0-6)
                    (row * 16 + col, 0.0)
                }
            };

            commands.spawn(SpriteSheetBundle {
                texture_atlas: game_assets.floor_tiles.clone(),
                sprite: TextureAtlasSprite {
                    index: sprite_index,
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(x_pos, y_pos, z_pos)),
                ..default()
            });
        }
    }
    }

