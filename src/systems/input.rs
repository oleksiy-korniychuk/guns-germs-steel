use bevy::prelude::*;
use bevy::input::mouse::{
    MouseScrollUnit,
    MouseWheel,
    MouseButton,
};

use crate::constants::*;
use crate::resources::{
    game_grid::SpatialGrid,
    camera::{CameraZoom, CameraPosition},
    ui_elements::{BandCenterVisualizationEnabled, LeftPanelState},
    band_center::{BandCenter, BandCenterMode},
};
use crate::components::components::*;


pub fn cursor_click_system(
    mut commands: Commands,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    creature_query: Query<(Entity, &Position, &Calories), With<CreatureMarker>>,
    plant_query: Query<(&Position, &FoodSource, &PlantMarker)>,
    grid: Res<SpatialGrid>,
    mut band_center: ResMut<BandCenter>,
    mut band_center_mode: ResMut<BandCenterMode>,
    mut panel_state: ResMut<LeftPanelState>,
    // Ensure only one creature has path visualization at a time
    creatures_with_viz: Query<Entity, (With<CreatureMarker>, With<PathVisualizationEnabled>)>,
) {
    // Only handle left mouse button clicks
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(world_position) = cast_cursor_position(windows, cameras) {
        let tile_x = (world_position.x / TILE_SIZE).floor() + GRID_WIDTH as f32 / 2.0;
        let tile_y = (world_position.y / TILE_SIZE).floor() + GRID_HEIGHT as f32 / 2.0;

        let position = Position { x: tile_x as i32, y: tile_y as i32 };

        // Check if we're clicking on valid grid coordinates
        if position.x >= 0 && position.x < GRID_WIDTH as i32 && position.y >= 0 && position.y < GRID_HEIGHT as i32 {
            let mut clicked_creature = false;
            
            if let Some(entities) = grid.0.get(&position) {
                for entity in entities.iter() {
                    // Handle creature clicks - toggle path visualization
                    if let Ok((creature_entity, position, calories)) = creature_query.get(*entity) {
                        clicked_creature = true;
                        info!("Clicked creature - Entity: {:?}, Position: {:?}, Calories: {:?}", creature_entity, position, calories);

                        // Select creature and ensure exclusive path visualization
                        // Remove viz from any previously visualized creatures
                        for other in creatures_with_viz.iter() {
                            if other != creature_entity {
                                commands.entity(other).remove::<PathVisualizationEnabled>();
                            }
                        }

                        *panel_state = LeftPanelState::Creature(creature_entity);
                        commands.entity(creature_entity).insert(PathVisualizationEnabled);
                        info!("Selected creature {:?}", creature_entity);
                    }
                    
                    // Still log plant info for debugging
                    if let Ok((position, food_source, plant_marker)) = plant_query.get(*entity) {
                        info!("Clicked plant - Entity: {:?}, Position: {:?}, Nutrition: {:?}, PlantType: {:?}", entity, position, food_source.nutrition_value, plant_marker.plant_type);
                    }
                }
            }
            
            // If we didn't click on a creature, set band center to manual mode at this position
            if !clicked_creature {
                *panel_state = LeftPanelState::None;
                *band_center_mode = BandCenterMode::Manual(position);
                band_center.0 = position;
                info!("Set band center to manual mode at position: {:?}", position);
            }
        }
    }
}

pub fn clear_selection_on_escape_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel_state: ResMut<LeftPanelState>,
    mut creatures_with_viz: Query<(Entity, &mut PathVisualizationEnabled), With<CreatureMarker>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if let LeftPanelState::Creature(entity) = *panel_state {
            *panel_state = LeftPanelState::None;
            // Remove visualization from the previously selected creature
            if creatures_with_viz.get_mut(entity).is_ok() {
                commands.entity(entity).remove::<PathVisualizationEnabled>();
            }
        }
    }
}

pub fn camera_zoom_system(
    mut commands: Commands,
    mut scroll_evr: EventReader<MouseWheel>,
    mut camera_zoom: ResMut<CameraZoom>,
    camera_query: Query<Entity, With<Camera2d>>,
    windows: Query<&Window>,
) {
    for ev in scroll_evr.read() {
        let zoom_delta = match ev.unit {
            MouseScrollUnit::Line => ev.y * ZOOM_SPEED * camera_zoom.0,
            MouseScrollUnit::Pixel => ev.y * ZOOM_SPEED * 0.01 * camera_zoom.0,
        };
        
        let max_zoom = if let Ok(window) = windows.single() {
            let map_width = GRID_WIDTH as f32 * TILE_SIZE;
            let map_height = GRID_HEIGHT as f32 * TILE_SIZE;
            
            let scale_for_width = map_width / window.width();
            let scale_for_height = map_height / window.height();
            let max_zoom_out = scale_for_width.max(scale_for_height);
            
            max_zoom_out
        } else {
            5.0
        };
        
        // Update zoom level
        camera_zoom.0 = (camera_zoom.0 - zoom_delta).clamp(MIN_ZOOM, max_zoom);
        
        // Apply zoom to camera
        if let Ok(camera_entity) = camera_query.single() {
            commands.entity(camera_entity).insert(Projection::from(OrthographicProjection {
                scale: camera_zoom.0,
                ..OrthographicProjection::default_2d()
            }));
        }
    }
}

pub fn camera_pan_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_position: ResMut<CameraPosition>,
    camera_query: Query<Entity, With<Camera2d>>,
    time: Res<Time>,
    camera_zoom: Res<CameraZoom>,
    windows: Query<&Window>,
) {
    let mut pan_direction = Vec2::ZERO;
    
    if keys.pressed(KeyCode::KeyW) {
        pan_direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        pan_direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        pan_direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        pan_direction.x += 1.0;
    }
    
    if pan_direction != Vec2::ZERO {
        // Normalize the direction to prevent faster diagonal movement
        pan_direction = pan_direction.normalize();
        
        // Scale pan speed by zoom level so panning feels consistent
        let pan_speed = CAMERA_PAN_SPEED * camera_zoom.0 * time.delta_secs();
        let new_position = camera_position.0 + pan_direction * pan_speed;
        
        // Calculate map boundaries
        let map_half_width = GRID_WIDTH as f32 * TILE_SIZE / 2.0;
        let map_half_height = GRID_HEIGHT as f32 * TILE_SIZE / 2.0;
        
        // Calculate viewport size based on zoom and window size
        if let Ok(window) = windows.single() {
            let viewport_half_width = window.width() * camera_zoom.0 / 2.0;
            let viewport_half_height = window.height() * camera_zoom.0 / 2.0;
            
            // If viewport is larger than map, center camera and don't allow panning
            if viewport_half_width >= map_half_width || viewport_half_height >= map_half_height {
                camera_position.0 = Vec2::ZERO;
            } else {
                // Calculate bounds that keep the viewport within the map
                let min_x = -map_half_width + viewport_half_width;
                let max_x = map_half_width - viewport_half_width;
                let min_y = -map_half_height + viewport_half_height;
                let max_y = map_half_height - viewport_half_height;
                
                // Apply boundary constraints
                camera_position.0.x = new_position.x.clamp(min_x, max_x);
                camera_position.0.y = new_position.y.clamp(min_y, max_y);
            }
        } else {
            // Fallback - just apply the movement without bounds
            camera_position.0 = new_position;
        };
        
        // Apply the new position to the camera
        if let Ok(camera_entity) = camera_query.single() {
            commands.entity(camera_entity).insert(Transform::from_translation(
                camera_position.0.extend(0.0)
            ));
        }
    }
}


pub fn band_center_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut viz_enabled: ResMut<BandCenterVisualizationEnabled>,
) {
    if keys.just_pressed(KeyCode::KeyB) {
        viz_enabled.0 = !viz_enabled.0;
        info!("Band center visualization toggled: {}", viz_enabled.0);
    }
}

// --- Helper Functions ---

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