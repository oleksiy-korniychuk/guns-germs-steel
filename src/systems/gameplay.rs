use bevy::prelude::*;
use crate::components::components::*;
use crate::resources::{
    game_grid::{
        SpatialGrid,
    },
    tick_count::TickCount,
};


pub fn spatial_grid_system(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Position)>, // Query for ALL entities with a position
) {
    grid.0.clear();
    for (entity, pos) in query.iter() {
        grid.0.entry(Position { x: pos.x, y: pos.y }).or_default().push(entity);
    }
}

pub fn tick_counter_system(mut tick_count: ResMut<TickCount>) {
    tick_count.0 += 1;
}
