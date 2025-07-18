use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::components::Position;

#[derive(Resource)]
pub struct GameGrid {
    pub tiles: Vec<Vec<Tile>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileKind {
    Empty
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub kind: TileKind,
}

#[derive(Resource, Default)]
pub struct SpatialGrid(pub HashMap<Position, Vec<Entity>>);