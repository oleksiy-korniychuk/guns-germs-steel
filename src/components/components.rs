use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug)]
pub struct Calories {
    pub current: i32,
    pub max: i32,
}

#[derive(Component, Debug, PartialEq)]
pub enum FsmState {
    Wandering,
    Traveling,
    Eating (EatingProgress)
}

#[derive(Debug, PartialEq)]
pub struct EatingProgress {
    pub progress: u32,
    pub max_progress: u32,
}

#[derive(Component, Debug)]
pub struct Goals {
    pub list: Vec<Goal>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Goal {
    Eat,
}

#[derive(Component, Debug)]
pub struct Target(pub Option<Position>);

#[derive(Component)]
pub struct MoveTo {
    pub destination: Position,
}

// --- Markers ---

#[derive(Component)]
pub struct CreatureMarker;

#[derive(Component)]
pub struct PlantMarker;

#[derive(Component)]
pub struct TileMarker;

#[derive(Component)]
pub struct Edible;

#[derive(Component)]
pub struct Harvestable;

#[derive(Component)]
pub struct TickText;