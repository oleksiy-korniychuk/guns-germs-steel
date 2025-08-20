use bevy::prelude::*;
use rand::Rng;
use crate::components::components::*;
use crate::resources::{
    game_grid::{
        SpatialGrid,
        GameGrid,
        TileKind,
    },
    band_center::BandCenter,
};
use crate::constants::*;
use std::collections::HashSet;
use pathfinding::prelude::astar;

#[derive(Event)]
pub struct NavigationFailed { pub entity: Entity, pub destination: Position }


// --- Intent-Driven Systems ---
pub fn goal_selection_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &Calories, &Position), (
        With<CreatureMarker>,
        Without<WantsToEat>,
        Without<WantsToIdle>,
        Without<WantsToProcreate>,
        Without<WantsToReturnToBand>,
        Without<ActionTravelTo>,
        Without<ActionEat>,
        Without<ActivePath>,
        Without<OutsideBandRadius>,
        Without<RequiresAt>,
    )>,
    pregnant_query: Query<(Entity, &mut Pregnant)>,
    band_center: Res<BandCenter>,
) {
    for (entity, calories, pos) in creature_query.iter() {
        let is_hungry = calories.current < (calories.max as f32 * 0.5) as i32;
        let is_outside_band_radius = is_outside_band_radius(*pos, band_center.0);
        

        if is_outside_band_radius {
            commands.entity(entity).insert(WantsToReturnToBand);
        } else if is_hungry {
            commands.entity(entity).insert(WantsToEat);
        } else if !pregnant_query.get(entity).is_ok() && calories.current >= (calories.max as f32 * 0.75) as i32 {
            commands.entity(entity).insert(WantsToProcreate);
        } else {
            commands.entity(entity).insert(WantsToIdle);
        }
    }
}

pub fn perform_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Position, &mut ActivePath, &mut Calories)>,
) {
    for (entity, mut pos, mut active_path, mut calories) in query.iter_mut() {
        if !active_path.nodes.is_empty() {
            let next_pos = active_path.nodes.remove(0);
            *pos = next_pos;
            calories.current -= MOVE_COST;
        }
        
        if active_path.nodes.is_empty() {
            commands.entity(entity).remove::<ActivePath>();
            // If there is a positional requirement and no longer moving, clear travel intent
            commands.entity(entity).remove::<ActionTravelTo>();
        }
    }
}

pub fn perform_eat_system(
    mut commands: Commands,
    mut creature_query: Query<(Entity, &Position, &mut Calories, &mut ActionEat), (With<CreatureMarker>, Without<ActivePath>)>,
    plant_query: Query<(&Position, &FoodSource), (With<PlantMarker>, With<Harvestable>, With<Edible>, Without<CreatureMarker>)>,
) {
    let mut plants_being_eaten = HashSet::new();
    
    for (creature_entity, creature_pos, mut creature_calories, mut eat_action) in creature_query.iter_mut() {
        if let Ok((plant_pos, plant_food)) = plant_query.get(eat_action.target_entity) {
            if *creature_pos == *plant_pos {
                // Check if another creature is already eating this plant this tick
                if plants_being_eaten.contains(&eat_action.target_entity) {
                    // Collision detected, reset this creature's intent
                    commands.entity(creature_entity)
                        .remove::<ActionEat>()
                        .remove::<RequiresAt>();
                    continue;
                }
                
                // Mark this plant as being eaten this tick
                plants_being_eaten.insert(eat_action.target_entity);
                
                eat_action.progress += 1;
                creature_calories.current -= WORK_COST;
                
                if eat_action.progress >= eat_action.max_progress {
                    creature_calories.current += plant_food.nutrition_value;
                    commands.entity(eat_action.target_entity).despawn();
                    commands.entity(creature_entity)
                        .remove::<ActionEat>()
                        .remove::<RequiresAt>();
                }
            }
        } else {
            // Target doesn't exist anymore, reset to searching
            commands.entity(creature_entity)
                .remove::<ActionEat>()
                .remove::<RequiresAt>()
                .insert(WantsToEat);
        }
    }
}

pub fn calorie_burn_system(mut query: Query<&mut Calories, With<CreatureMarker>>) {
    for mut calories in query.iter_mut() {
        calories.current -= LIVE_COST;
    }
}

pub fn death_system(mut commands: Commands, query: Query<(Entity, &Calories)>) {
    for (entity, calories) in query.iter() {
        if calories.current <= 0 {
            commands.entity(entity).despawn(); // now also takes care of despawn child entities
        }
    }
}

pub fn find_food_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &Position), (With<CreatureMarker>, With<WantsToEat>)>,
    food_query: Query<(), (With<PlantMarker>, With<Harvestable>, With<Edible>)>,
    food_pos_query: Query<&Position, (With<PlantMarker>, With<Harvestable>, With<Edible>)>,
    spatial_grid: Res<SpatialGrid>,
) {
    let mut targeted_plants = HashSet::new();
    
    for (creature_entity, creature_pos) in creature_query.iter() {
        if let Some(food_entity) = find_closest_available_food(&spatial_grid, &food_query, *creature_pos, &targeted_plants) {
            if let Ok(food_pos) = food_pos_query.get(food_entity) {
                // Mark this plant as targeted
                targeted_plants.insert(food_entity);
                
                commands.entity(creature_entity)
                    .remove::<WantsToEat>()
                    .insert(ActionEat { 
                        target_entity: food_entity,
                        progress: 0,
                        max_progress: 3,
                    })
                    .insert(RequiresAt { position: *food_pos, radius: 0 });
            }
        } else {
            commands.entity(creature_entity).remove::<WantsToEat>();
        }
    }
}

pub fn idle_goal_selection_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &Position, &Calories), (With<CreatureMarker>, With<WantsToIdle>)>,
) {
    let mut rng = rand::rng();
    for (entity, pos, calories) in creature_query.iter() {
        if calories.current < calories.max {
            commands.entity(entity).remove::<WantsToIdle>();
            commands.entity(entity).insert(WantsToEat);
        } else {
            let mut new_pos = *pos;
            match rng.random_range(0..5) {
                0 => new_pos.y = (new_pos.y - 1).max(0),
                1 => new_pos.y = (new_pos.y + 1).min(GRID_HEIGHT as i32 - 1),
                2 => new_pos.x = (new_pos.x - 1).max(0),
                3 => new_pos.x = (new_pos.x + 1).min(GRID_WIDTH as i32 - 1),
                _ => {} // Stay put
            }
            
            commands.entity(entity)
                .remove::<WantsToIdle>()
                .insert(ActionTravelTo { destination: new_pos });
        }
    }
}

pub fn procreation_system(
    mut commands: Commands,
    mut creature_query: Query<(Entity, &mut Calories), (With<CreatureMarker>, With<WantsToProcreate>)>,
) {
    for (entity, mut calories) in creature_query.iter_mut() {
        calories.current -= PREGNANT_COST;
        commands.entity(entity).insert(Pregnant { progress: 0, max_progress: HUMAN_PREGNANCY_DURATION });
        commands.entity(entity).remove::<WantsToProcreate>();
    }
}

pub fn pregnancy_system(
    mut commands: Commands,
    mut creature_query: Query<(Entity, &mut Pregnant, &Position), (With<CreatureMarker>, With<Pregnant>)>,
) {
    for (entity, mut pregnant, pos) in creature_query.iter_mut() {
        pregnant.progress += 1;
        if pregnant.progress >= pregnant.max_progress {
            let mut spawn_position = *pos;

            // Check all adjacent positions
            if pos.y > 0 { 
                spawn_position = Position { x: pos.x, y: pos.y - 1 }; 
            } else if pos.y < GRID_HEIGHT as i32 - 1 { 
                spawn_position = Position { x: pos.x, y: pos.y + 1 }; 
            } else if pos.x > 0 { 
                spawn_position = Position { x: pos.x - 1, y: pos.y }; 
            } else if pos.x < GRID_WIDTH as i32 - 1 { 
                spawn_position = Position { x: pos.x + 1, y: pos.y }; 
            }

            commands.spawn((
                CreatureMarker,
                Position { x: spawn_position.x, y: spawn_position.y },
                Calories { current: (HUMAN_MAX_CALORIES / 2) as i32, max: HUMAN_MAX_CALORIES },
            ));

            commands.entity(entity).remove::<Pregnant>();
        }
    }
}

// A* pathfinding system that calculates optimal paths using the game grid
pub fn pathfinding_system(
    mut commands: Commands,
    query: Query<(Entity, &Position, &ActionTravelTo), Without<ActivePath>>,
    game_grid: Res<GameGrid>,
    mut nav_failed: EventWriter<NavigationFailed>,
) {
    for (entity, current_pos, travel_action) in query.iter() {
        let destination = travel_action.destination;
        
        if *current_pos == destination {
            commands.entity(entity).remove::<ActionTravelTo>();
        } else {
            // Calculate A* path from current position to destination
            if let Some(path) = calculate_astar_path(*current_pos, destination, &game_grid) {
                // Remove the first position (current position) from the path
                let mut nodes = path;
                if !nodes.is_empty() && nodes[0] == *current_pos {
                    nodes.remove(0);
                }
                
                // If we have valid moves, create ActivePath
                if !nodes.is_empty() {
                    commands.entity(entity).insert(ActivePath { nodes });
                } else {
                    // Already at destination
                    commands.entity(entity).remove::<ActionTravelTo>();
                }
            } else {
                // No path found, handle navigation failure without touching unrelated actions
                commands.entity(entity).remove::<ActionTravelTo>();
                nav_failed.write(NavigationFailed { entity, destination });
                warn!("No path found from {:?} to {:?}", current_pos, destination);
            }
        }
    }
}

pub fn return_to_band_system(
    mut commands: Commands,
    creature_query: Query<Entity, (With<CreatureMarker>, With<WantsToReturnToBand>)>,
    band_center: Res<BandCenter>,
) {
    for entity in creature_query.iter() {
        commands.entity(entity).insert(OutsideBandRadius);
        commands.entity(entity).remove::<WantsToReturnToBand>();
        commands.entity(entity).insert(ActionTravelTo { destination: band_center.0 });
    }
}

pub fn check_if_returned_to_band_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &Position, Option<&ActionEat>), (With<CreatureMarker>, With<OutsideBandRadius>)>,
    band_center: Res<BandCenter>,
) {
    for (entity, pos, maybe_eat) in creature_query.iter() {
        if !is_outside_band_radius(*pos, band_center.0) {
            commands.entity(entity).remove::<OutsideBandRadius>();
            commands.entity(entity).remove::<ActionTravelTo>();
            commands.entity(entity).remove::<ActivePath>();
            // Clear positional requirements that were forcing a return
            commands.entity(entity).remove::<RequiresAt>();
            // If they were in the middle of eating, cancel and let planner re-assign
            if maybe_eat.is_some() {
                commands.entity(entity).remove::<ActionEat>().insert(WantsToEat);
            }
        }
    }
}

// Ensures navigation exists for actions that require proximity
pub fn action_preconditions_system(
    mut commands: Commands,
    query: Query<(Entity, &Position, &RequiresAt, Option<&ActionTravelTo>, Option<&ActivePath>)>,
) {
    for (entity, pos, req, maybe_travel, maybe_path) in query.iter() {
        let distance = (pos.x - req.position.x).abs() + (pos.y - req.position.y).abs();
        let at_target = distance <= req.radius;
        if !at_target && maybe_travel.is_none() && maybe_path.is_none() {
            commands.entity(entity).insert(ActionTravelTo { destination: req.position });
        }
        // Optional: if already at target but still has travel intent, clear it
        if at_target && maybe_travel.is_some() {
            commands.entity(entity).remove::<ActionTravelTo>();
        }
    }
}

// React to navigation failures and reset actions as needed
pub fn action_failure_resolution_system(
    mut commands: Commands,
    mut nav_failed: EventReader<NavigationFailed>,
    has_eat: Query<(), With<ActionEat>>,
    has_requires: Query<(), With<RequiresAt>>,
) {
    for ev in nav_failed.read() {
        let _destination = ev.destination; // access to avoid unused-field warning
        let entity = ev.entity;
        let eat_present = has_eat.get(entity).is_ok();
        let req_present = has_requires.get(entity).is_ok();
        if eat_present || req_present {
            commands.entity(entity)
                .remove::<ActionEat>()
                .remove::<RequiresAt>()
                .remove::<ActionTravelTo>()
                .remove::<ActivePath>()
                .insert(WantsToEat);
        }
    }
}

pub fn update_band_center_system(
    creature_query: Query<&Position, With<CreatureMarker>>,
    mut band_center: ResMut<BandCenter>,
) {
    if creature_query.is_empty() {
        return;
    }
    
    let mut new_band_center = Position { x: 0, y: 0 };
    for pos in creature_query.iter() {
        new_band_center.x += pos.x;
        new_band_center.y += pos.y;
    }
    new_band_center.x /= creature_query.iter().count() as i32;
    new_band_center.y /= creature_query.iter().count() as i32;
    band_center.0 = new_band_center;
}

// --- Helper Functions ---

// A* pathfinding function that uses the game grid for tile costs
fn calculate_astar_path(
    start: Position,
    end: Position,
    game_grid: &GameGrid,
) -> Option<Vec<Position>> {
    let result = astar(
        &start,
        |p| {
            // Generate all possible neighbors (4-directional movement)
            let neighbors = vec![
                Position { x: p.x + 1, y: p.y },
                Position { x: p.x - 1, y: p.y },
                Position { x: p.x, y: p.y + 1 },
                Position { x: p.x, y: p.y - 1 },
            ];

            neighbors.into_iter()
                .filter_map(|neighbor_pos| {
                    // Check if position is within bounds
                    if neighbor_pos.x < 0 || neighbor_pos.x >= GRID_WIDTH as i32 ||
                       neighbor_pos.y < 0 || neighbor_pos.y >= GRID_HEIGHT as i32 {
                        return None;
                    }

                    // Get the tile at this position
                    let tile = &game_grid.tiles[neighbor_pos.y as usize][neighbor_pos.x as usize];
                    
                    // Calculate cost based on tile type and move_cost
                    let cost = match tile.kind {
                        TileKind::Empty => 10,  // Standard cost for empty tiles
                        TileKind::Dirt => tile.move_cost as u32,  // Use tile's move_cost
                        TileKind::Water => {
                            // Water is very expensive to traverse (simulating need for boats/swimming)
                            tile.move_cost as u32 * 10
                        }
                    };

                    // If cost is reasonable, include this neighbor
                    if cost <= 1000 {  // Prevent unreasonably high costs
                        Some((neighbor_pos, cost))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        },
        |p| {
            // Manhattan distance heuristic
            ((p.x - end.x).abs() + (p.y - end.y).abs()) as u32
        },
        |p| *p == end  // Success condition
    );

    // Extract just the path from the result
    result.map(|(path, _cost)| path)
}

// Optimized search function using a spatial grid.
fn find_closest_available_food(
    grid: &Res<SpatialGrid>,
    food_query: &Query<(), (With<PlantMarker>, With<Harvestable>, With<Edible>)>,
    start_pos: Position,
    targeted_plants: &HashSet<Entity>,
) -> Option<Entity> {
    for radius in 0i32..100 {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }

                let check_pos = Position { x: start_pos.x + dx, y: start_pos.y + dy };

                if let Some(entities_in_cell) = grid.0.get(&check_pos) {
                    for &entity in entities_in_cell {
                        if food_query.get(entity).is_ok() && !targeted_plants.contains(&entity) {
                            return Some(entity);
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn is_outside_band_radius(
    pos: Position,
    band_center: Position,
) -> bool {
    let dx = band_center.x - pos.x;
    let dy = band_center.y - pos.y;
    if dx * dx + dy * dy > BAND_RADIUS * BAND_RADIUS {
        return true;
    }
    false
}
