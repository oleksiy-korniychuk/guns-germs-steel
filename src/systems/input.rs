use bevy::prelude::*;

use crate::constants::*;
use crate::resources::game_grid::SpatialGrid;
use crate::components::components::*;


pub fn cursor_click_system(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    creature_query: Query<(&Position, &Calories), With<CreatureMarker>>,
    plant_query: Query<(&Position, &FoodSource, &PlantMarker)>,
    grid: Res<SpatialGrid>,
) {
    if let Some(world_position) = cast_cursor_position(windows, cameras) {
        let tile_x = (world_position.x / TILE_SIZE).floor() + GRID_WIDTH as f32 / 2.0;
        let tile_y = (world_position.y / TILE_SIZE).floor() + GRID_HEIGHT as f32 / 2.0;

        let position = Position { x: tile_x as i32, y: tile_y as i32 };

        if let Some(entities) = grid.0.get(&position) {
            for entity in entities.iter() {
                if let Ok((position, calories)) = creature_query.get(*entity) {
                    info!("Entity: {:?}, Position: {:?}, Calories: {:?}", entity, position, calories);
                }
                if let Ok((position, food_source, plant_marker)) = plant_query.get(*entity) {
                    info!("Entity: {:?}, Position: {:?}, Nutrition: {:?}, PlantType: {:?}", entity, position, food_source.nutrition_value, plant_marker.plant_type);
                }
            }
        }

    }
}

pub fn cast_cursor_position(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    if let Ok((camery, position)) = cameras.single() {
        return windows
            .single()
            .map(|window| window.cursor_position())
            .unwrap_or_default()
            .map(|cursor| camery.viewport_to_world_2d(position, cursor))
            .map(|result| result.unwrap());
    }
    None
}