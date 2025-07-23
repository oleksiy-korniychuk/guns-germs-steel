use bevy::prelude::*;
use rand::Rng;
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
};
use crate::components::components::*;

pub fn setup_system(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    // --- Resource Setup ---
    let mut rng = rand::rng();
    let grid_tiles = vec![vec![Tile { kind: TileKind::Empty }; GRID_WIDTH]; GRID_HEIGHT];

    commands.insert_resource(GameGrid { tiles: grid_tiles });
    commands.insert_resource(SpatialGrid::default());
    commands.insert_resource(TickCount::default());
    commands.insert_resource(PopulationCount::default());
    commands.insert_resource(BandCenter(Position { x: 0, y: 0 }));

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
} 