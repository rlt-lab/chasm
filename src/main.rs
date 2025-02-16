use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::render::camera::ScalingMode;
use bevy::window::MonitorSelection;
const TILE_SIZE: f32 = 32.0;
use bevy::sprite::TextureAtlas;
mod components;
mod map;
use map::{MAP_WIDTH, MAP_HEIGHT};
mod rendering;
mod input;
mod ui;

use input::{InputState, Player, GridPosition};

#[derive(Resource)]
struct GameAssets {
    character_sheet: Handle<TextureAtlas>,
    floor_tiles: Handle<TextureAtlas>,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    InGame,
    Paused,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Chasm".into(),
                resolution: (
                    MAP_WIDTH as f32 * TILE_SIZE,
                    MAP_HEIGHT as f32 * TILE_SIZE,
                ).into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                resizable: false,
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .add_state::<GameState>()
        .init_resource::<InputState>()
        .add_systems(Startup, setup)
        .add_systems(PostStartup, spawn_game_world)
        .add_systems(Update, (
            input::handle_input,
            input::move_player
        ).run_if(in_state(GameState::InGame)))
        .add_systems(Update, update_sprite_positions
            .after(input::move_player)
            .run_if(in_state(GameState::InGame)))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Camera
    let mut camera = Camera2dBundle::default();
    camera.transform.translation = Vec3::new(
        (MAP_WIDTH as f32 * TILE_SIZE) / 2.0,
        (MAP_HEIGHT as f32 * TILE_SIZE) / 2.0,
        999.9
    );
    camera.projection.scaling_mode = ScalingMode::Fixed {
        width: MAP_WIDTH as f32 * TILE_SIZE,
        height: MAP_HEIGHT as f32 * TILE_SIZE,
    };
    commands.spawn(camera);
    
    // Load character sprite sheet
    let character_handle = asset_server.load("sprites/rogues.png");
    let character_atlas = TextureAtlas::from_grid(
        character_handle,
        Vec2::new(32.0, 32.0),  // assuming 32x32 sprites
        16, 16,                 // 16x16 grid
        None, None
    );
    let character_atlas_handle = texture_atlases.add(character_atlas);

    // Load tiles sprite sheet
    let tiles_handle = asset_server.load("sprites/tiles.png");
    let tiles_atlas = TextureAtlas::from_grid(
        tiles_handle,
        Vec2::new(32.0, 32.0),  // assuming 32x32 sprites
        16, 16,                 // 16x16 grid
        None, None
    );
    let tiles_atlas_handle = texture_atlases.add(tiles_atlas);

    // Store the atlas handles as a resource
    commands.insert_resource(GameAssets {
        character_sheet: character_atlas_handle,
        floor_tiles: tiles_atlas_handle,
    });
}

fn spawn_game_world(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    let map = map::TileMap::new();
    commands.insert_resource(map.clone());
    rendering::spawn_tiles(&mut commands, &map, &game_assets);
    map::spawn_map(&mut commands);
    
    // Spawn player
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: game_assets.character_sheet.clone(),
            sprite: TextureAtlasSprite {
                index: 4 * 16 + 1, // Position 5.b for wizard
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 1.0)
                .with_scale(Vec3::splat(1.0)),
            ..default()
        },
        Player,
        GridPosition { x: (MAP_WIDTH / 2) as i32, y: (MAP_HEIGHT / 2) as i32 },
    ));
}

fn update_sprite_positions(
    mut query: Query<(&GridPosition, &mut Transform), With<Player>>,
) {
    for (grid_pos, mut transform) in query.iter_mut() {
        transform.translation.x = grid_pos.x as f32 * TILE_SIZE + (TILE_SIZE / 2.0);
        transform.translation.y = grid_pos.y as f32 * TILE_SIZE + (TILE_SIZE / 2.0);
        transform.translation.z = 1.0;  // Keep player above tiles
    }
}
