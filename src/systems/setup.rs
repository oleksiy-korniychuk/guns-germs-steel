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
    ui_elements::{TickCount, PopulationCount},
    seed::WorldSeed,
    camera::CameraZoom,
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
    let world_seed = generate_seed();
    let grid_tiles = generate_height_map(world_seed);
    // Find dirt tiles near map center for creatures
    let creature_positions = find_dirt_near_center(&grid_tiles);

    // --- Spawning Initial Entities ---
    // Spawn Creatures
    commands.spawn((
        CreatureMarker,
        creature_positions.0,
        Calories { current: HUMAN_MAX_CALORIES, max: HUMAN_MAX_CALORIES },
    ));
    commands.spawn((
        CreatureMarker,
        creature_positions.1,
        Calories { current: HUMAN_MAX_CALORIES, max: HUMAN_MAX_CALORIES },
    ));

    // Spawn Plants using noise-based wheat generation
    generate_wheat_patches(&mut commands, &grid_tiles, world_seed);

    commands.insert_resource(GameGrid { tiles: grid_tiles });
    commands.insert_resource(SpatialGrid::default());
    commands.insert_resource(TickCount::default());
    commands.insert_resource(PopulationCount::default());
    commands.insert_resource(BandCenter(Position { x: 0, y: 0 }));
    commands.insert_resource(WorldSeed(world_seed));
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
    let mut map = vec![vec![Tile { kind: TileKind::Empty, move_cost: 0 }; GRID_WIDTH]; GRID_HEIGHT];
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let nx = x as f64 * SCALE;
            let ny = y as f64 * SCALE;
            let raw_height = perlin.get([nx, ny]); // Value in [-1, 1]
            let height = ((raw_height + 1.0) / 2.0) as f32; // Normalize to [0,1]
            if height < WATER_LEVEL {
                map[y][x] = Tile { kind: TileKind::Water, move_cost: 100 };
            } else {
                map[y][x] = Tile { kind: TileKind::Dirt, move_cost: 1 };
            }
        }
    }
    map
}

fn find_dirt_near_center(grid: &Vec<Vec<Tile>>) -> (Position, Position) {
    let center_x = (GRID_WIDTH / 2) as i32;
    let center_y = (GRID_HEIGHT / 2) as i32;
    let mut dirt_positions = Vec::new();
    
    'outer: for radius in 0..20 {
        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                if radius == 0 || dx.abs() == radius as i32 || dy.abs() == radius as i32 {
                    let (x, y) = (center_x + dx, center_y + dy);
                    
                    if (0..GRID_WIDTH as i32).contains(&x) && (0..GRID_HEIGHT as i32).contains(&y) 
                        && grid[y as usize][x as usize].kind == TileKind::Dirt {
                        dirt_positions.push(Position { x, y });
                        if dirt_positions.len() >= 2 {
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
    
    match dirt_positions.len() {
        0 => (Position { x: 0, y: 0 }, Position { x: 0, y: 0 }),
        1 => (dirt_positions[0], dirt_positions[0]),
        _ => (dirt_positions[0], dirt_positions[1]),
    }
}

fn generate_wheat_patches(commands: &mut Commands, grid_tiles: &Vec<Vec<Tile>>, world_seed: u32) {
    // Use a different seed offset for wheat generation to create different patterns
    let wheat_seed = world_seed.wrapping_add(12345);
    let wheat_noise = Perlin::new(wheat_seed);
    
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            // Only place wheat on dirt tiles
            if grid_tiles[y][x].kind == TileKind::Dirt {
                let nx = x as f64 * WHEAT_SCALE;
                let ny = y as f64 * WHEAT_SCALE;
                let wheat_noise_value = wheat_noise.get([nx, ny]); // Value in [-1, 1]
                let normalized_wheat = ((wheat_noise_value + 1.0) / 2.0) as f32; // Normalize to [0,1]
                
                // Primary wheat patch determination
                if normalized_wheat > WHEAT_THRESHOLD {
                    commands.spawn((
                        PlantMarker { plant_type: PlantType::Wheat },
                        Position { x: x as i32, y: y as i32 },
                        FoodSource { nutrition_value: WHEAT_NUTRIENTS },
                        Harvestable,
                        Edible,
                    ));
                }
            }
        }
    }
}
