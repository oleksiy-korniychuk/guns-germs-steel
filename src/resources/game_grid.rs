use bevy::prelude::*;

#[derive(Resource)]
pub struct GameGrid {
    pub tiles: Vec<Vec<Tile>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileKind {
    Empty,
    CerealGrass {
        calories: i32,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub kind: TileKind,
}

pub trait Consumable {
    fn consume(&mut self) -> i32;
}

impl Consumable for Tile {
    fn consume(&mut self) -> i32 {
        match self.kind {
            TileKind::CerealGrass { calories} => {
                self.kind = TileKind::Empty;
                calories
            },
            TileKind::Empty => 0,
        }
    }
}