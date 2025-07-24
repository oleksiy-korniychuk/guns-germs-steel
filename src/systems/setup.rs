use bevy::prelude::*;
use rand::Rng;
use rand_pcg::Pcg32;
use noise::{NoiseFn, Perlin};

use crate::constants::*;
use crate::resources::{
    game_grid::{
        GameGrid,
        TileKind,
        Tile,
        SpatialGrid,
    },
    band_center::BandCenter,
    tick_count::TickCount,
    population_count::PopulationCount,
    seed::WorldSeed,
    camera_zoom::CameraZoom,
};
use crate::components::components::*;

pub fn setup_system(mut commands: Commands, camera_zoom: Res<CameraZoom>) {
    commands.spawn((
        Camera2d::default(),
        Projection::from(OrthographicProjection {
            scale: camera_zoom.0,
            ..OrthographicProjection::default_2d()
        }),
    ));

    // --- Resource Setup ---
    let mut rng = rand::rng();
    let world_seed = generate_seed();
    let grid_tiles = generate_height_map(world_seed);

    commands.insert_resource(GameGrid { tiles: grid_tiles });
    commands.insert_resource(SpatialGrid::default());
    commands.insert_resource(TickCount::default());
    commands.insert_resource(PopulationCount::default());
    commands.insert_resource(BandCenter(Position { x: 0, y: 0 }));
    commands.insert_resource(WorldSeed(world_seed));

    // --- Spawning Initial Entities ---
    // Spawn Creatures
    commands.spawn((
        CreatureMarker,
        Position { x: rng.random_range(0..GRID_WIDTH as i32), y: rng.random_range(0..GRID_HEIGHT as i32) },
        Calories { current: HUMAN_MAX_CALORIES, max: HUMAN_MAX_CALORIES },
    ));
    commands.spawn((
        CreatureMarker,
        Position { x: rng.random_range(0..GRID_WIDTH as i32), y: rng.random_range(0..GRID_HEIGHT as i32) },
        Calories { current: HUMAN_MAX_CALORIES, max: HUMAN_MAX_CALORIES },
    ));
    // Spawn Plants
    for _ in 0..STARTING_GRASS_COUNT {
        let x = rng.random_range(0..GRID_WIDTH);
        let y = rng.random_range(0..GRID_HEIGHT);
        commands.spawn((
            PlantMarker { plant_type: PlantType::Wheat },
            Position { x: x as i32, y: y as i32 },
            FoodSource { nutrition_value: WHEAT_NUTRIENTS },
            Harvestable,
            Edible,
        ));
    }
}

pub fn setup_visualization_system(
    mut commands: Commands,
    grid: Res<GameGrid>,
    world_seed: Res<WorldSeed>,
) {
    // --- Draw the Grid ---
    // We spawn a sprite for each tile only once
    for (y, row) in grid.tiles.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let (color, image) = match tile.kind {
                TileKind::Empty => {
                    if (x + y) % 2 == 0 {
                        (Color::srgb(0.4, 0.4, 0.4), default())
                    } else {
                        (Color::srgb(0.5, 0.5, 0.5), default())
                    }
                }
                TileKind::Dirt => {
                    (Color::srgb(0.5, 0.5, 0.5), default())
                }
                TileKind::Water => {
                    (Color::srgb(0.0, 0.0, 1.0), default())
                }
            };

            commands.spawn((
                TileMarker,
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    image,
                    ..default()
                },
                Transform::from_xyz(
                    x as f32 * TILE_SIZE - (GRID_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
                    y as f32 * TILE_SIZE - (GRID_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
                    0.0, // Z-index for 2D layering
                ),
                Position { x: x as i32, y: y as i32 }, // Give the sprite a grid position
            ));
        }
    }
    
    // --- Draw the UI/UX Elements ---
    commands.spawn((
        TickText,
        Text::new("Tick: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
    commands.spawn((
        PopulationText,
        Text::new("Population: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(30.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
    info!("World seed: {}", world_seed.0);
}

// --- Helper Functions ---

fn generate_seed() -> u32 {
    let mut rng = Pcg32::new(
        rand::rng().random_range(0..u64::MAX),
        rand::rng().random_range(0..u64::MAX),
    );
    rng.random_range(0..u32::MAX)
}

fn generate_height_map(seed: u32) -> Vec<Vec<Tile>> {
    let perlin = Perlin::new(seed);
    let mut map = vec![vec![Tile { kind: TileKind::Empty }; GRID_WIDTH]; GRID_HEIGHT];
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let nx = x as f64 * SCALE;
            let ny = y as f64 * SCALE;
            let raw_height = perlin.get([nx, ny]); // Value in [-1, 1]
            let height = ((raw_height + 1.0) / 2.0) as f32; // Normalize to [0,1]
            if height < WATER_LEVEL {
                map[y][x] = Tile { kind: TileKind::Water };
            } else {
                map[y][x] = Tile { kind: TileKind::Dirt };
            }
        }
    }
    map
}
