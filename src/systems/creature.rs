use bevy::prelude::*;
use crate::components::components::*;
use crate::resources::{
    game_grid::{
        SpatialGrid,
    },
};
use crate::constants::*;
use std::collections::HashSet;


// --- Intent-Driven Systems ---
pub fn goal_selection_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &Calories), (
        With<CreatureMarker>,
        Without<WantsToEat>,
        Without<WantsToIdle>,
        Without<ActionTravelTo>,
        Without<ActionEat>,
        Without<ActivePath>,
        Without<WantsToProcreate>,
    )>,
) {
    for (entity, calories) in creature_query.iter() {
        let is_hungry = calories.current < (calories.max as f32 * 0.5) as i32;
        
        if is_hungry {
            commands.entity(entity).insert(WantsToEat);
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
                        .remove::<ActionEat>();
                    continue;
                }
                
                // Mark this plant as being eaten this tick
                plants_being_eaten.insert(eat_action.target_entity);
                
                eat_action.progress += 1;
                creature_calories.current -= WORK_COST;
                
                if eat_action.progress >= eat_action.max_progress {
                    creature_calories.current += plant_food.nutrition_value;
                    commands.entity(eat_action.target_entity).despawn();
                    commands.entity(creature_entity).remove::<ActionEat>();
                }
            }
        } else {
            // Target doesn't exist anymore, reset to searching
            commands.entity(creature_entity)
                .remove::<ActionEat>();
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
            commands.entity(entity).despawn();
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
                    .insert(ActionTravelTo { destination: *food_pos })
                    .insert(ActionEat { 
                        target_entity: food_entity,
                        progress: 0,
                        max_progress: 3,
                    });
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
    // let mut rng = rand::rng();
    for (entity, pos, calories) in creature_query.iter() {
        if calories.current < calories.max {
            commands.entity(entity).remove::<WantsToIdle>();
            commands.entity(entity).insert(WantsToEat);
        } else {
            commands.entity(entity).remove::<WantsToIdle>();
            commands.entity(entity).insert(WantsToProcreate);
        }
        // else {
        //     let mut new_pos = *pos;
        //     match rng.random_range(0..5) {
        //         0 => new_pos.y = (new_pos.y - 1).max(0),
        //         1 => new_pos.y = (new_pos.y + 1).min(GRID_HEIGHT as i32 - 1),
        //         2 => new_pos.x = (new_pos.x - 1).max(0),
        //         3 => new_pos.x = (new_pos.x + 1).min(GRID_WIDTH as i32 - 1),
        //         _ => {} // Stay put
        //     }
            
        //     commands.entity(entity)
        //         .remove::<WantsToIdle>()
        //         .insert(ActionTravelTo { destination: new_pos });
        // }
    }
}

pub fn procreation_system(
    mut commands: Commands,
    mut creature_query: Query<(Entity, &Position, &mut Calories), (With<CreatureMarker>, With<WantsToProcreate>)>,
) {
    for (entity, pos, mut calories) in creature_query.iter_mut() {
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
            Calories { current: 50, max: 100 },
        ));

        calories.current -= (calories.max as f32 * 0.5) as i32;
        commands.entity(entity).remove::<WantsToProcreate>();
    }
}


// Simple pathfinding system that only calculates the next step (For now)
pub fn pathfinding_system(
    mut commands: Commands,
    query: Query<(Entity, &Position, &ActionTravelTo), Without<ActivePath>>,
) {
    for (entity, current_pos, travel_action) in query.iter() {
        let destination = travel_action.destination;
        
        if *current_pos == destination {
            // Reached destination, remove ActionTravelTo
            commands.entity(entity).remove::<ActionTravelTo>();
        } else {
            // Not at destination yet, calculate next step and keep ActionTravelTo
            let mut next_pos = *current_pos;
            let dx = destination.x - current_pos.x;
            let dy = destination.y - current_pos.y;

            if dx.abs() > dy.abs() {
                next_pos.x += dx.signum();
            } else {
                next_pos.y += dy.signum();
            }
            
            commands.entity(entity).insert(ActivePath { nodes: vec![next_pos] });
        }
    }
}

// --- Helper Functions ---

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
