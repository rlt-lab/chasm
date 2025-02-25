use bevy::prelude::*;
use crate::map::{TileMap, TileType, MAP_WIDTH, MAP_HEIGHT};
#[derive(Component, Default)]
pub struct TileVisibility {
    pub visible: bool,
    pub previously_seen: bool,
}

#[derive(Component, Default)]
pub struct PlayerVisibility {
    pub range: f32,
}

#[derive(Resource, Default)]
pub struct VisibilityMap {
    pub visible_tiles: Vec<Vec<bool>>,
    pub previously_seen: Vec<Vec<bool>>,
}

pub fn setup_visibility_map(mut commands: Commands) {
    let visibility_map = VisibilityMap {
        visible_tiles: vec![vec![false; MAP_WIDTH]; MAP_HEIGHT],
        previously_seen: vec![vec![false; MAP_WIDTH]; MAP_HEIGHT],
    };
    commands.insert_resource(visibility_map);
}

pub fn update_tile_visibility(
    visibility_map: Res<VisibilityMap>,
    mut query: Query<&mut TileVisibility>,
) {
    for (i, mut tile_vis) in query.iter_mut().enumerate() {
        let x = i % MAP_WIDTH;
        let y = i / MAP_WIDTH;
        
        // Add bounds checking to prevent index out of bounds errors
        if y < MAP_HEIGHT && x < MAP_WIDTH {
            tile_vis.visible = visibility_map.visible_tiles[y][x];
            if tile_vis.visible {
                tile_vis.previously_seen = true;
            }
        } else {
            // Default to not visible if out of bounds
            tile_vis.visible = false;
        }
    }
}

pub fn update_visibility(
    mut visibility_map: ResMut<VisibilityMap>,
    query: Query<(&Transform, &PlayerVisibility)>,
    map: Res<TileMap>,
) {
    // Store current visible tiles in previously_seen
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            if visibility_map.visible_tiles[y][x] {
                visibility_map.previously_seen[y][x] = true;
            }
            visibility_map.visible_tiles[y][x] = false;
        }
    }

    for (transform, visibility) in query.iter() {
        let pos = transform.translation;
        let player_pos = (
            (pos.x / 32.0).round() as i32,
            (pos.y / 32.0).round() as i32
        );
        
        // Cast rays in a 360-degree arc
        for angle in 0..360 {
            let rad = angle as f32 * 0.0174533;
            let end_x = player_pos.0 + (visibility.range * rad.cos()) as i32;
            let end_y = player_pos.1 + (visibility.range * rad.sin()) as i32;
            cast_ray(player_pos.0, player_pos.1, end_x, end_y, &mut visibility_map, &map);
        }
    }
}


fn cast_ray(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    visibility_map: &mut VisibilityMap,
    map: &TileMap,
) {
    let points = bresenham_line(start_x, start_y, end_x, end_y);
    
    for point in points {
        if point.0 >= 0 && point.0 < MAP_WIDTH as i32 && 
        point.1 >= 0 && point.1 < MAP_HEIGHT as i32 {
            visibility_map.visible_tiles[point.1 as usize][point.0 as usize] = true;
            
            // Stop if we hit a wall
            if map.tiles[point.1 as usize][point.0 as usize] == TileType::Wall {
                break;
            }
        } else {
            break;
        }
    }
}

fn blocks_sight(x: i32, y: i32, map: &TileMap) -> bool {
    if x < 0 || x >= MAP_WIDTH as i32 || y < 0 || y >= MAP_HEIGHT as i32 {
        return true;
    }
    map.tiles[y as usize][x as usize] == TileType::Wall
}

fn bresenham_line(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let mut x = x0;
    let mut y = y0;
    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        points.push((x, y));
        if x == x1 && y == y1 { break; }
        let e2 = 2 * error;
        if e2 >= dy {
            if x == x1 { break; }
            error += dy;
            x += step_x;
        }
        if e2 <= dx {
            if y == y1 { break; }
            error += dx;
            y += step_y;
        }
    }
    points
}

