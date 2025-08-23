use bevy::prelude::{Resource, Entity};

#[derive(Resource, Default)]
pub struct TickCount(pub u32);

#[derive(Resource, Default)]
pub struct PopulationCount(pub u32);

#[derive(Resource, Default)]
pub struct BandCenterVisualizationEnabled(pub bool);

#[derive(Resource, Debug, Clone, Copy)]
pub enum LeftPanelState {
    None,
    Creature(Entity),
}

impl Default for LeftPanelState {
    fn default() -> Self { Self::None }
}