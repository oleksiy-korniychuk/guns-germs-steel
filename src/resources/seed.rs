use bevy::prelude::Resource;
    
#[derive(Resource, Default)]
pub struct WorldSeed(pub u32);