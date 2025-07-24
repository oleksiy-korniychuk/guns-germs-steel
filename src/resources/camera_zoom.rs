use bevy::prelude::*;
use crate::constants::DEFAULT_ZOOM;

#[derive(Resource)]
pub struct CameraZoom(pub f32);

impl Default for CameraZoom {
    fn default() -> Self {
        Self(DEFAULT_ZOOM)
    }
}