use bevy::prelude::Resource;
    
#[derive(Resource, Default)]
pub struct PopulationCount(pub u32);