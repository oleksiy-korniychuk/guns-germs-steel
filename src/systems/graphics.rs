use bevy::prelude::*;
use crate::resources::{
    ui_elements::{TickCount, PopulationCount, LeftPanelState},
    band_center::BandCenter,
    ui_elements::BandCenterVisualizationEnabled,
};
use crate::components::components::*;
use crate::constants::*;
use rand::Rng;

pub fn spawn_creature_visuals_system(
    mut commands: Commands,
    query: Query<(Entity, &Position), (With<CreatureMarker>, Added<Position>)>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::rng();
    
    for (entity, pos) in query.iter() {
        commands.entity(entity).insert(
            Sprite {
                color: Color::srgb(0.0, 1.0, 0.0), // Default color
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                image: asset_server.load("sprites/human_v2.png"),
                ..default()
            }
        );
        commands.entity(entity).insert(
            Transform::from_xyz(
                pos.x as f32 * TILE_SIZE - (GRID_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
                pos.y as f32 * TILE_SIZE - (GRID_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
                2.0, // Higher Z-index to be on top of tiles
            )
        );

        // Create a child entity for the headband
        let headband_entity = commands.spawn((
            Sprite {
                color: HEADBAND_COLORS[rng.random_range(0..HEADBAND_COLORS.len())],
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                image: asset_server.load("sprites/human_headband_v2.png"),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1), // Relative to parent, slightly higher Z
        )).id();

        // Make the headband a child of the creature
        commands.entity(entity).add_child(headband_entity);
    }
}

pub fn spawn_plant_visuals_system(
    mut commands: Commands,
    query: Query<(Entity, &Position), (With<PlantMarker>, Added<Position>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, pos) in query.iter() {
        commands.entity(entity).insert(
            Sprite {
                color: Color::srgb(0.0, 1.0, 0.0), // Default color
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                image: asset_server.load("sprites/wheat.png"),
                ..default()
            }
        );
        commands.entity(entity).insert(
            Transform::from_xyz(
                pos.x as f32 * TILE_SIZE - (GRID_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
                pos.y as f32 * TILE_SIZE - (GRID_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
                1.0, // Higher Z-index to be on top of tiles
            )
        );
    }
}

// System to update the visual position of creatures when their grid Position changes
pub fn update_creature_position_visuals_system(
    mut query: Query<(&mut Transform, &Position), With<CreatureMarker>>,
) {
    for (mut transform, pos) in query.iter_mut() {
        transform.translation.x = pos.x as f32 * TILE_SIZE - (GRID_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
        transform.translation.y = pos.y as f32 * TILE_SIZE - (GRID_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    }
}

// System to update creature color based on health
pub fn update_creature_color_system(mut query: Query<(&mut Sprite, &Calories), With<CreatureMarker>>) {
    for (mut sprite, cals) in query.iter_mut() {
        sprite.color = if cals.current >= cals.max {
            Color::srgb(0.0, 1.0, 0.0)
        } else if cals.current >= (cals.max as f32 / 2.0) as i32 {
            Color::srgb(1.0, 1.0, 0.0)
        } else if cals.current >= (cals.max as f32 / 4.0) as i32 {
            Color::srgb(1.0, 0.5, 0.0)
        } else {
            Color::srgb(1.0, 0.0, 0.0)
        };
    }
}

// Path visualization system - creates visual markers for active paths
pub fn path_visualization_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &ActivePath), (With<CreatureMarker>, With<PathVisualizationEnabled>)>,
    existing_path_markers: Query<Entity, With<PathMarker>>,
) {
    // Clean up existing path markers first
    for marker_entity in existing_path_markers.iter() {
        commands.entity(marker_entity).despawn();
    }
    
    // Create new path markers for creatures with visualization enabled
    for (creature_entity, active_path) in creature_query.iter() {
        for (index, &path_node) in active_path.nodes.iter().enumerate() {
            // Calculate world position from grid position
            let world_x = (path_node.x as f32 - GRID_WIDTH as f32 / 2.0) * TILE_SIZE;
            let world_y = (path_node.y as f32 - GRID_HEIGHT as f32 / 2.0) * TILE_SIZE;
            
            // Create a visual marker for this path node
            commands.spawn((
                Sprite {
                    color: if index == 0 { 
                        Color::srgb(1.0, 1.0, 0.0) // Yellow for next step
                    } else { 
                        Color::srgb(0.0, 1.0, 1.0) // Cyan for future steps
                    },
                    custom_size: Some(Vec2::new(TILE_SIZE * 0.3, TILE_SIZE * 0.3)),
                    ..default()
                },
                Transform::from_xyz(world_x + TILE_SIZE/2.0, world_y + TILE_SIZE/2.0, 3.0),
                PathMarker {
                    creature_entity
                },
            ));
        }
    }
}

// Cleanup system to remove path visualization when creatures die or lose ActivePath
pub fn cleanup_path_visualization_system(
    mut commands: Commands,
    path_markers: Query<(Entity, &PathMarker)>,
    creatures_with_paths: Query<(), (With<CreatureMarker>, With<ActivePath>, With<PathVisualizationEnabled>)>,
) {
    for (marker_entity, path_marker) in path_markers.iter() {
        // If the creature no longer exists or doesn't have the required components, remove the marker
        if creatures_with_paths.get(path_marker.creature_entity).is_err() {
            commands.entity(marker_entity).despawn();
        }
    }
}

pub fn band_center_visualization_system(
    mut commands: Commands,
    band_center: Res<BandCenter>,
    viz_enabled: Res<BandCenterVisualizationEnabled>,
    existing_center_markers: Query<Entity, With<BandCenterMarker>>,
    existing_circle_markers: Query<Entity, With<BandCircleMarker>>,
) {
    // Clean up existing band center markers
    for marker_entity in existing_center_markers.iter() {
        commands.entity(marker_entity).despawn();
    }
    
    // Clean up existing band circle markers
    for marker_entity in existing_circle_markers.iter() {
        commands.entity(marker_entity).despawn();
    }
    
    // If visualization is enabled, spawn new markers at the current band center
    if viz_enabled.0 {
        let world_x = (band_center.0.x as f32 - GRID_WIDTH as f32 / 2.0) * TILE_SIZE;
        let world_y = (band_center.0.y as f32 - GRID_HEIGHT as f32 / 2.0) * TILE_SIZE;
        let center_world_pos = Vec3::new(world_x + TILE_SIZE/2.0, world_y + TILE_SIZE/2.0, 4.0);
        
        // Spawn the red center dot
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0), // Red color
                custom_size: Some(Vec2::new(TILE_SIZE / 3.0, TILE_SIZE / 3.0)), // About a third of tile size
                ..default()
            },
            Transform::from_translation(center_world_pos),
            BandCenterMarker,
        ));
        
        // Spawn the cyan circle outline
        let circle_radius = BAND_RADIUS as f32 * TILE_SIZE; // Convert grid units to world units
        commands.spawn((
            Sprite {
                color: Color::srgba(0.0, 1.0, 1.0, 0.3), // Cyan with transparency
                custom_size: Some(Vec2::new(circle_radius * 2.0, circle_radius * 2.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(center_world_pos.x, center_world_pos.y, 3.0)), // Lower Z-index so it's behind the dot
            BandCircleMarker,
        ));
    }
}

pub fn update_tick_text_system(
    tick_count: Res<TickCount>,
    mut query: Query<&mut Text, With<TickText>>,
) {
    if tick_count.is_changed() {
        if let Ok(mut text) = query.single_mut() {
            text.clear();
            text.push_str(&format!("Tick: {}", tick_count.0));
        }
    }
}

pub fn update_population_text_system(
    population_count: Res<PopulationCount>,
    mut query: Query<&mut Text, With<PopulationText>>,
) {
    if population_count.is_changed() {
        if let Ok(mut text) = query.single_mut() {
            text.clear();
            text.push_str(&format!("Population: {}", population_count.0));
        }
    }
}

pub fn update_selected_panel_system(
    panel_state: Res<LeftPanelState>,
    mut root_query: Query<&mut Node, With<SelectedPanelRoot>>,
    mut text_nodes: Query<
        (&mut Text, Option<&SelectedEntityIdText>, Option<&SelectedCaloriesText>, Option<&SelectedPregnancyText>),
        Without<SelectedPanelRoot>
    >,
    creatures: Query<(Entity, Option<&Calories>, Option<&Pregnant>), With<CreatureMarker>>,
) {
    if let Ok(mut node) = root_query.single_mut() {
        node.display = match *panel_state {
            LeftPanelState::Creature(_) => Display::Flex,
            LeftPanelState::None => Display::None,
        };
    }

    let mut entity_line: Option<String> = None;
    let mut calories_line: Option<String> = None;
    let mut pregnancy_line: Option<String> = None;

    if let LeftPanelState::Creature(entity) = *panel_state {
        if let Ok((cre_entity, calories_opt, pregnant_opt)) = creatures.get(entity) {
            entity_line = Some(format!("Entity: {:?}", cre_entity));
            if let Some(cal) = calories_opt {
                calories_line = Some(format!("Calories: {}/{}", cal.current, cal.max));
            }
            pregnancy_line = Some(match pregnant_opt {
                Some(p) => format!("Pregnancy: yes {}/{}", p.progress, p.max_progress),
                None => "Pregnancy: no".to_string(),
            });
        } else {
            // Selected entity no longer exists; hide panel
            entity_line = None;
            calories_line = None;
            pregnancy_line = None;
        }
    }

    for (mut text, is_id, is_cal, is_preg) in text_nodes.iter_mut() {
        if is_id.is_some() {
            let new_value = entity_line.as_deref().unwrap_or("Entity: -");
            // Avoid unnecessary text mutations
            if text.0 != new_value {
                text.clear();
                text.push_str(new_value);
            }
        } else if is_cal.is_some() {
            let new_value = calories_line.as_deref().unwrap_or("Calories: -/-");
            if text.0 != new_value {
                text.clear();
                text.push_str(new_value);
            }
        } else if is_preg.is_some() {
            let new_value = pregnancy_line.as_deref().unwrap_or("Pregnancy: no");
            if text.0 != new_value {
                text.clear();
                text.push_str(new_value);
            }
        }
    }
}
