use bevy::prelude::Color;

// --- Game Constants ---
pub const GRID_WIDTH: usize = 700;
pub const GRID_HEIGHT: usize = 400;
pub const TILE_SIZE: f32 = 32.0;
pub const TICK_RATE_HZ: f64 = 2.0;

// --- Window/Camera Constants ---
pub const DEFAULT_WINDOW_WIDTH: f32 = 1200.0;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 800.0;
pub const DEFAULT_ZOOM: f32 = 1.0;
pub const MIN_ZOOM: f32 = 0.1;
pub const ZOOM_SPEED: f32 = 0.1;
pub const CAMERA_PAN_SPEED: f32 = 400.0;

// --- World Constants ---
pub const WATER_LEVEL: f32 = 0.3; // Tiles below this are lakes
pub const SCALE: f64 = 0.02;      // Controls how zoomed in/out the noise is
pub const STARTING_GRASS_COUNT: i32 = 5000;

// --- Creature Constants ---
pub const MOVE_COST: i32 = 2;
pub const LIVE_COST: i32 = 1;
pub const WORK_COST: i32 = 20;
pub const PREGNANT_COST: i32 = 1000;
pub const HUMAN_MAX_CALORIES: i32 = 2500;
pub const HUMAN_PREGNANCY_DURATION: u32 = 75;
pub const BAND_RADIUS: i32 = 10;

// --- Plant Constants ---
pub const WHEAT_NUTRIENTS: i32 = 1000;


// --- Visual Constants ---
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
    