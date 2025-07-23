use bevy::prelude::Resource;
use crate::components::components::Position;

#[derive(Resource)]
pub struct BandCenter(pub Position);
