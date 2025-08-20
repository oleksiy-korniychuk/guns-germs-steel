use bevy::prelude::Resource;
use crate::components::components::Position;

#[derive(Resource)]
pub struct BandCenter(pub Position);

#[derive(Resource)]
pub enum BandCenterMode {
    Auto,
    Manual(Position),
}