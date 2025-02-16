use bevy::prelude::*;
use crate::map::{TileMap, TileType, MAP_SIZE};
use crate::GameAssets;
use rand::Rng;
const TILE_SIZE: f32 = 32.0;

pub fn spawn_tiles(
    commands: &mut Commands,
    map: &TileMap,
    game_assets: &Res<GameAssets>,
) {
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let (x_pos, y_pos) = (
                x as f32 * TILE_SIZE,
                y as f32 * TILE_SIZE + 100.0  // Offset up by 100 pixels to leave room for message box
            );

            let (sprite_index, z_pos) = match map.tiles[x][y] {
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

