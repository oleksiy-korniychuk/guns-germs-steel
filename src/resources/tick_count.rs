use bevy::prelude::Resource;
    
#[derive(Resource, Default)]
pub struct TickCount(pub u32);