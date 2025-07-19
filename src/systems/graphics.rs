use bevy::prelude::*;
use crate::resources::{
    game_grid::{
        GameGrid,
        TileKind,
    },
    tick_count::TickCount,
};
use crate::components::components::*;
use crate::constants::*;
use rand::Rng;

pub fn spawn_creature_visuals_system(
    mut commands: Commands,
    query: Query<(Entity, &Position), (With<CreatureMarker>, Added<Position>)>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::rng();
    
    for (entity, pos) in query.iter() {
        let headband_color = Color::srgb(
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0), 
            rng.random_range(0.0..1.0)
        );
        
        commands.entity(entity).insert(
            Sprite {
                color: Color::srgb(1.0, 1.0, 0.0), // Default color
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                image: asset_server.load("sprites/human_v2.png"),
                ..default()
            }
        );
        commands.entity(entity).insert(
            Transform::from_xyz(
                pos.x as f32 * TILE_SIZE - (GRID_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
                pos.y as f32 * TILE_SIZE - (GRID_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
                2.0, // Higher Z-index to be on top of tiles
            )
        );

        // Create a child entity for the headband
        let headband_entity = commands.spawn((
            Sprite {
                color: headband_color, // This color stays the same
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                image: asset_server.load("sprites/human_headband_v2.png"),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1), // Relative to parent, slightly higher Z
        )).id();

        // Make the headband a child of the creature
        commands.entity(entity).add_child(headband_entity);
    }
}

pub fn spawn_plant_visuals_system(
    mut commands: Commands,
    query: Query<(Entity, &Position), (With<PlantMarker>, Added<Position>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, pos) in query.iter() {
        commands.entity(entity).insert(
            Sprite {
                color: Color::srgb(0.0, 1.0, 0.0), // Default color
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                image: asset_server.load("sprites/wheat.png"),
                ..default()
            }
        );
        commands.entity(entity).insert(
            Transform::from_xyz(
                pos.x as f32 * TILE_SIZE - (GRID_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
                pos.y as f32 * TILE_SIZE - (GRID_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
                1.0, // Higher Z-index to be on top of tiles
            )
        );
    }
}

// System to update the visual position of creatures when their grid Position changes
pub fn update_creature_position_system(
    mut query: Query<(&mut Transform, &Position), With<CreatureMarker>>,
) {
    for (mut transform, pos) in query.iter_mut() {
        transform.translation.x = pos.x as f32 * TILE_SIZE - (GRID_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
        transform.translation.y = pos.y as f32 * TILE_SIZE - (GRID_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    }
}

// System to update creature color based on health
pub fn update_creature_color_system(mut query: Query<(&mut Sprite, &Calories), With<CreatureMarker>>) {
    for (mut sprite, cals) in query.iter_mut() {
        sprite.color = if cals.current >= cals.max {
            Color::srgb(0.0, 1.0, 0.0)
        } else if cals.current >= (cals.max as f32 / 2.0) as i32 {
            Color::srgb(1.0, 1.0, 0.0)
        } else if cals.current >= (cals.max as f32 / 4.0) as i32 {
            Color::srgb(1.0, 0.5, 0.0)
        } else {
            Color::srgb(1.0, 0.0, 0.0)
        };
    }
}

// System to update the tick counter text
pub fn update_tick_text_system(
    tick_count: Res<TickCount>,
    mut query: Query<&mut Text, With<TickText>>,
) {
    if tick_count.is_changed() {
        for mut text in query.iter_mut() {
            text.clear();
            text.push_str(&format!("Tick: {}", tick_count.0));
        }
    }
}

// System to update tile colors when they change (e.g., grass is eaten)
pub fn update_tile_visuals_system(
    grid: Res<GameGrid>,
    mut query: Query<(&mut Sprite, &Position), With<TileMarker>>,
) {
    if grid.is_changed() {
        for (mut sprite, pos) in query.iter_mut() {
            let tile = &grid.tiles[pos.y as usize][pos.x as usize];
            (sprite.color, sprite.image) = match tile.kind {
                TileKind::Empty => {
                    if (pos.x + pos.y) % 2 == 0 {
                        (Color::srgb(0.4, 0.4, 0.4), default())
                    } else {
                        (Color::srgb(0.5, 0.5, 0.5), default())
                    }
                }
            };
        }
    }
} 