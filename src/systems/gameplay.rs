use bevy::prelude::*;
use rand::Rng;
use crate::components::components::*;
use crate::resources::{
    game_grid::{
        GameGrid,
        TileKind,
        Consumable,
    },
    tick_count::TickCount,
};
use crate::constants::*;

pub fn fsm_decision_system(
    mut query: Query<(&mut FsmState, &Calories, &mut Target, &Position), With<CreatureTag>>,
    grid: Res<GameGrid>,
) {
    for (mut fsm, calories, mut target, pos) in query.iter_mut() {
        let is_hungry = calories.current < (calories.max as f32 / 2.0) as i32;

        if is_hungry && *fsm == FsmState::Wandering {
            if let Some(food_pos) = find_closest_food(&grid, pos) {
                *fsm = FsmState::SeekingFood;
                target.0 = Some(food_pos);
            }
        } else if !is_hungry && *fsm != FsmState::Wandering {
             // If not hungry anymore, go back to wandering
            if let FsmState::SeekingFood = *fsm {
                *fsm = FsmState::Wandering;
                target.0 = None;
            }
        }
    }
}

pub fn wandering_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Position, &FsmState), With<CreatureTag>>,
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
                commands.entity(entity).insert(WantsToMove { destination: new_pos });
            }
        }
    }
}

pub fn seeking_food_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Position, &mut FsmState, &mut Target), With<CreatureTag>>,
    grid: Res<GameGrid>,
) {
    for (entity, pos, mut fsm, mut target) in query.iter_mut() {
        if *fsm == FsmState::SeekingFood {
            if let Some(target_pos) = target.0 {
                // Have we arrived?
                if *pos == target_pos {
                    let tile = &grid.tiles[target_pos.y as usize][target_pos.x as usize];
                    if let TileKind::CerealGrass { .. } = tile.kind {
                        *fsm = FsmState::Eating { progress: 0, max_progress: 3 };
                    } else {
                        // Food is gone, go back to wandering.
                        *fsm = FsmState::Wandering;
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
                commands.entity(entity).insert(WantsToMove { destination: new_pos });
            } else {
                // No target? Go back to wandering.
                *fsm = FsmState::Wandering;
            }
        }
    }
}

pub fn eating_system(
    mut query: Query<(&Position, &mut FsmState, &mut Calories), With<CreatureTag>>,
    mut grid: ResMut<GameGrid>,
) {
    for (pos, mut fsm, mut calories) in query.iter_mut() {
        if let FsmState::Eating { progress, max_progress } = *fsm {
            let new_progress = progress + 1;
            calories.current -= WORK_COST;

            if new_progress >= max_progress {
                let current_tile = &mut grid.tiles[pos.y as usize][pos.x as usize];
                let calories_gained = current_tile.consume();
                calories.current += calories_gained;
                
                // Finished eating, go back to wandering
                *fsm = FsmState::Wandering;
            } else {
                *fsm = FsmState::Eating { progress: new_progress, max_progress };
            }
        }
    }
}

pub fn movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Position, &WantsToMove, &mut Calories)>,
) {
    for (entity, mut pos, wants_to_move, mut calories) in query.iter_mut() {
        if *pos != wants_to_move.destination {
            *pos = wants_to_move.destination;
            calories.current -= MOVE_COST;
        }
        // The intent has been handled, so we remove the component.
        commands.entity(entity).remove::<WantsToMove>();
    }
}

pub fn calorie_burn_system(mut query: Query<&mut Calories, With<CreatureTag>>) {
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
pub fn find_closest_food(grid: &Res<GameGrid>, position: &Position) -> Option<Position> {
    let mut closest_food: Option<(Position, i32)> = None;

    for (y, row) in grid.tiles.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            if let TileKind::CerealGrass { .. } = tile.kind {
                let food_pos = Position { x: x as i32, y: y as i32 };
                let dist = (position.x - food_pos.x).abs() + (position.y - food_pos.y).abs(); // Manhattan distance

                if let Some((_, min_dist)) = closest_food {
                    if dist < min_dist {
                        closest_food = Some((food_pos, dist));
                    }
                } else {
                    closest_food = Some((food_pos, dist));
                }
            }
        }
    }
    closest_food.map(|(pos, _)| pos)
} 