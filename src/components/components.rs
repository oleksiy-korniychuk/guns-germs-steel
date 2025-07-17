use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
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
    SeekingFood,
    Eating {
        progress: u32,
        max_progress: u32,
    },
}

#[derive(Component, Debug)]
pub struct Target(pub Option<Position>);

#[derive(Component)]
pub struct WantsToMove {
    pub destination: Position,
}

// --- Tags ---

#[derive(Component)]
pub struct CreatureTag;

#[derive(Component)]
pub struct TileSprite;

#[derive(Component)]
pub struct TickText;