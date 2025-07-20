use bevy::prelude::*;
use rand::Rng;
use crate::components::components::*;
use crate::resources::{
    game_grid::{
        SpatialGrid,
    },
    tick_count::TickCount,
    population_count::PopulationCount,
};
use crate::constants::*;


pub fn spatial_grid_system(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Position), Without<TileMarker>>,
) {
    grid.0.clear();
    for (entity, pos) in query.iter() {
        grid.0.entry(Position { x: pos.x, y: pos.y }).or_default().push(entity);
    }
}

pub fn tick_counter_system(mut tick_count: ResMut<TickCount>) {
    tick_count.0 += 1;
}

pub fn population_counter_system(
    creature_query: Query<&CreatureMarker>,
    mut population_count: ResMut<PopulationCount>,
) {
    let population = creature_query.iter().count();
    population_count.0 = population as u32;
}

pub fn plant_propogation_system(
    mut commands: Commands,
    plant_query: Query<(&Position, &PlantMarker)>,
    grid: Res<SpatialGrid>,
) {
    for (pos, plant_marker) in plant_query.iter() {
        let spawn_plant = rand::rng().random_range(0..100) == 0; // 1% chance
        if spawn_plant {
            let mut empty_neighbors = Vec::new();
            
            // Check all 8 surrounding positions
            for x in -1..=1 {
                for y in -1..=1 {
                    let neighbor_x = pos.x + x;
                    let neighbor_y = pos.y + y;
                    
                    // Check if position is within grid bounds
                    if neighbor_x >= 0 && neighbor_x < GRID_WIDTH as i32 &&
                       neighbor_y >= 0 && neighbor_y < GRID_HEIGHT as i32 {
                        let neighbor_pos = Position { x: neighbor_x, y: neighbor_y };
                        
                        // Check if this position is empty (no entities at this position)
                        if !grid.0.contains_key(&neighbor_pos) {
                            empty_neighbors.push(neighbor_pos);
                        }
                    }
                }
            }
            
            // If there are empty neighbors, pick one at random and spawn a plant there
            if !empty_neighbors.is_empty() {
                let random_index = rand::rng().random_range(0..empty_neighbors.len());
                let spawn_pos = empty_neighbors[random_index];
                
                commands.spawn((
                    PlantMarker { plant_type: plant_marker.plant_type },
                    Position { x: spawn_pos.x, y: spawn_pos.y },
                    FoodSource { nutrition_value: 20 },
                    Harvestable,
                    Edible,
                ));
            }
        }
    }
}
