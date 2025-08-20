use bevy::prelude::Resource;

#[derive(Resource, Default)]
pub struct TickCount(pub u32);

#[derive(Resource, Default)]
pub struct PopulationCount(pub u32);

#[derive(Resource, Default)]
pub struct BandCenterVisualizationEnabled(pub bool);
