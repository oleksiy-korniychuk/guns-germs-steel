use bevy::prelude::Color;

pub const GRID_WIDTH: usize = 40;
pub const GRID_HEIGHT: usize = 30;
pub const TILE_SIZE: f32 = 32.0;
pub const STARTING_GRASS_COUNT: i32 = 100;
pub const MOVE_COST: i32 = 1;
pub const LIVE_COST: i32 = 2;
pub const WORK_COST: i32 = 2;
pub const PREGNANT_COST: i32 = 100;
pub const TICK_RATE_HZ: f64 = 2.0;
pub const WHEAT_NUTRIENTS: i32 = 100;
pub const HUMAN_MAX_CALORIES: i32 = 250;
pub const HUMAN_PREGNANCY_DURATION: u32 = 75;

pub const HEADBAND_COLORS: [Color; 12] = [
        Color::srgb(1.0, 0.0, 0.0),     // Red
        Color::srgb(0.0, 1.0, 0.0),     // Green
        Color::srgb(0.0, 0.0, 1.0),     // Blue
        Color::srgb(1.0, 1.0, 0.0),     // Yellow
        Color::srgb(1.0, 0.0, 1.0),     // Magenta
        Color::srgb(0.0, 1.0, 1.0),     // Cyan
        Color::srgb(1.0, 0.5, 0.0),     // Orange
        Color::srgb(0.5, 0.0, 1.0),     // Purple
        Color::srgb(1.0, 1.0, 1.0),     // White
        Color::srgb(0.0, 0.0, 0.0),     // Black
        Color::srgb(0.5, 0.5, 0.5),     // Gray
        Color::srgb(1.0, 0.0, 0.5),     // Pink
    ];