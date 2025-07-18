use bevy::prelude::*;
use rand::Rng;
use crate::components::components::*;
use crate::resources::{
    game_grid::{
        SpatialGrid,
    },
    tick_count::TickCount,
};
use crate::constants::*;

pub fn fsm_decision_system(
    mut creature_query: Query<(&mut FsmState, &Calories, &mut Target, &Position, &mut Goals), With<CreatureMarker>>,
    food_query: Query<(), (With<PlantMarker>, With<Harvestable>, With<Edible>)>,
    spatial_grid: Res<SpatialGrid>,
) {
    for (mut fsm, calories, mut target, pos, mut goals) in creature_query.iter_mut() {
        let is_hungry = calories.current < (calories.max as f32 / 2.0) as i32;

        if is_hungry && !matches!(*fsm, FsmState::Traveling | FsmState::Eating { .. }) {
            if let Some(food_entity) = find_closest_food_entity(&spatial_grid, &food_query, *pos) {
                *fsm = FsmState::Traveling;
                goals.list.push(Goal::Eat);
                target.0 = Some(food_entity);
            }
        }
    }
}

pub fn wandering_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Position, &FsmState), With<CreatureMarker>>,
) {
    let mut rng = rand::rng();
    for (entity, pos, fsm) in query.iter_mut() {
        if *fsm == FsmState::Wandering {
            let mut new_pos = *pos;
            match rng.random_range(0..5) {
                0 => new_pos.y = (new_pos.y - 1).max(0),
                1 => new_pos.y = (new_pos.y + 1).min(GRID_HEIGHT as i32 - 1),
                2 => new_pos.x = (new_pos.x - 1).max(0),
                3 => new_pos.x = (new_pos.x + 1).min(GRID_WIDTH as i32 - 1),
                _ => {} // Stay put
            }
            // Instead of changing Position directly, we add a component to signal intent.
            if new_pos != *pos {
                commands.entity(entity).insert(MoveTo { destination: new_pos });
            }
        }
    }
}

pub fn traveling_system(
    mut commands: Commands,
    mut creature_query: Query<(Entity, &Position, &mut FsmState, &mut Target, &Goals), With<CreatureMarker>>,
    plant_query: Query<&Position, With<PlantMarker>>,
) {
    for (entity, pos, mut fsm, mut target, goals) in creature_query.iter_mut() {
        if *fsm == FsmState::Traveling {
            if let Some(target_entity) = target.0 {
                if let Ok(target_pos) = plant_query.get(target_entity) {
                    // Have we arrived?
                    if *pos == *target_pos {
                        if goals.list[0] == Goal::Eat {
                            *fsm = FsmState::Eating { progress: 0, max_progress: 3, entity: Some(target_entity) };
                            target.0 = None;
                        }
                        continue; // Done with this entity for this tick
                    }
                    // Move towards the target
                    let mut new_pos = *pos;
                    let dx = target_pos.x - pos.x;
                    let dy = target_pos.y - pos.y;
    
                    if dx.abs() > dy.abs() {
                        new_pos.x += dx.signum();
                    } else {
                        new_pos.y += dy.signum();
                    }
                    commands.entity(entity).insert(MoveTo { destination: new_pos });
                }

            } else {
                // No target? Go back to wandering.
                *fsm = FsmState::Wandering;
            }
        }
    }
}

pub fn eating_system(
    mut commands: Commands,
    mut creature_query: Query<(&mut FsmState, &mut Calories, &mut Goals), With<CreatureMarker>>,
    plant_query: Query<(Entity, &Calories), (With<PlantMarker>, With<Harvestable>, With<Edible>, Without<CreatureMarker>)>,
) {
    for (mut fsm, mut calories, mut goals) in creature_query.iter_mut() {
        if let FsmState::Eating { progress, max_progress, entity } = *fsm {
            let new_progress = progress + 1;
            calories.current -= WORK_COST;
            println!("Eating: {:?}", calories.current);
            println!("Progress: {:?}", new_progress);

            if new_progress >= max_progress {
                println!("Eating plant: {:?}", entity);
                if let Ok((plant_entity, plant_calories)) = plant_query.get(entity.unwrap()) {
                    println!("Eating plant: {:?}", plant_entity);
                    calories.current += plant_calories.current;
                    commands.entity(plant_entity).despawn();
                }
                
                // Finished eating, go back to wandering
                goals.list.remove(0);
                *fsm = FsmState::Wandering;
            } else {
                *fsm = FsmState::Eating { progress: new_progress, max_progress, entity };
            }
        }
    }
}

pub fn spatial_grid_system(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Position)>, // Query for ALL entities with a position
) {
    grid.0.clear();
    for (entity, pos) in query.iter() {
        grid.0.entry(Position { x: pos.x, y: pos.y }).or_default().push(entity);
    }
}

pub fn movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Position, &MoveTo, &mut Calories)>,
) {
    for (entity, mut pos, wants_to_move, mut calories) in query.iter_mut() {
        if *pos != wants_to_move.destination {
            *pos = wants_to_move.destination;
            calories.current -= MOVE_COST;
        }
        // The intent has been handled, so we remove the component.
        commands.entity(entity).remove::<MoveTo>();
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

pub fn tick_counter_system(mut tick_count: ResMut<TickCount>) {
    tick_count.0 += 1;
}

// --- Helper Functions ---
// Optimized search function using a spatial grid.
fn find_closest_food_entity(
    grid: &Res<SpatialGrid>,
    food_query: &Query<(), (With<PlantMarker>, With<Harvestable>, With<Edible>)>,
    start_pos: Position,
) -> Option<Entity> {
    // Search in an expanding spiral pattern for efficiency.
    // Start with a search radius of 0.
    for radius in 0i32..100 { // Limit search radius to avoid infinite loops
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                // Only check the cells on the perimeter of the current search box
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }

                let check_pos = Position { x: start_pos.x + dx, y: start_pos.y + dy };

                if let Some(entities_in_cell) = grid.0.get(&check_pos) {
                    for &entity in entities_in_cell {
                        // Check if the entity in the cell is actually food.
                        if food_query.get(entity).is_ok() {
                            return Some(entity); // Found the closest food!
                        }
                    }
                }
            }
        }
    }

    None // No food found within the search radius
}