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

#[derive(Component, Debug)]
pub struct FoodSource {
    pub nutrition_value: i32,
}

// --- Intent Components ---

#[derive(Component, Debug)]
pub struct WantsToEat;

#[derive(Component, Debug)]
pub struct WantsToIdle;

#[derive(Component, Debug)]
pub struct WantsToProcreate;

#[derive(Component, Debug)]
pub struct WantsToReturnToBand;

// --- Action Components ---

#[derive(Component, Debug)]
pub struct ActionTravelTo {
    pub destination: Position,
}

#[derive(Component, Debug)]
pub struct ActionEat {
    pub target_entity: Entity,
    pub progress: u32,
    pub max_progress: u32,
}

#[derive(Component, Debug)]
pub struct ActivePath {
    pub nodes: Vec<Position>,
}

#[derive(Component, Debug)]
pub struct OutsideBandRadius;

// --- Markers ---

#[derive(Component)]
pub struct CreatureMarker;

#[derive(Component, Debug)]
pub struct Pregnant {
    pub progress: u32,
    pub max_progress: u32,
}

#[derive(Component)]
pub struct PlantMarker {
    pub plant_type: PlantType,
}

#[derive(Component)]
pub struct TileMarker;

#[derive(Component)]
pub struct Edible;

#[derive(Component)]
pub struct Harvestable;

#[derive(Component, Debug)]
pub struct BeingConsumed {
    pub consumer_entity: Entity,
}

#[derive(Component)]
pub struct TickText;

#[derive(Component)]
pub struct PopulationText;

#[derive(Component)]
pub struct BandCenterMarker;

#[derive(Component)]
pub struct BandCircleMarker;

#[derive(Component)]
pub struct PathVisualizationEnabled;

#[derive(Component, Debug)]
pub struct PathMarker {
    pub creature_entity: Entity
}

// --- UI: Selection Panel Markers ---
#[derive(Component)]
pub struct SelectedPanelRoot;

#[derive(Component)]
pub struct SelectedEntityIdText;

#[derive(Component)]
pub struct SelectedCaloriesText;

#[derive(Component)]
pub struct SelectedPregnancyText;

// --- Enums ---
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlantType {
    Wheat
}

// --- Events ---
#[derive(Event, Debug)]
pub struct FoodTargetInvalidated {
    pub creature_entity: Entity,
}
