use bevy::prelude::{Resource, Entity};
use std::collections::HashMap;
use crate::components::components::Position;

#[derive(Resource)]
pub struct GameGrid {
    pub tiles: Vec<Vec<Tile>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileKind {
    Empty,
    Dirt,
    Water,
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub kind: TileKind,
    pub move_cost: i32,
}

#[derive(Resource, Default)]
pub struct SpatialGrid(pub HashMap<Position, Vec<Entity>>);