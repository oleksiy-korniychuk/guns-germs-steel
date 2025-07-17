use bevy::prelude::*;
use bevy::app::AppExit;
use rand::Rng;

const GRID_WIDTH: usize = 40;
const GRID_HEIGHT: usize = 30;
const TILE_SIZE: f32 = 32.0;
const STARTING_GRASS_COUNT: i32 = 80;
const MOVE_COST: i32 = 1;
const LIVE_COST: i32 = 1;
const WORK_COST: i32 = 1;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Running,
    Paused,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Guns, Germs, and Steel!".into(),
                resolution: (
                    GRID_WIDTH as f32 * TILE_SIZE,
                    GRID_HEIGHT as f32 * TILE_SIZE,
                ).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_systems(
            Startup, 
            (
                setup_system,
                setup_visualization_system,
            ).chain(),
        )
        .add_systems(
            FixedUpdate,
            (
                fsm_decision_system,
                wandering_system,
                seeking_food_system,
                eating_system,
                movement_system,
                calorie_burn_system,
                death_system,
                tick_counter_system,
            ).chain().run_if(in_state(GameState::Running)),
        )
        .add_systems(
            Update, 
            (
                toggle_pause_system,
                exit_on_escape_system,
                spawn_creature_visuals_system,
                update_creature_color_system,
                update_creature_position_system,
                update_tick_text_system,
                update_tile_visuals_system,
            ).chain(),
        )
        .insert_resource(Time::<Fixed>::from_hz(1.0))
        .run();
}

// --- Tiles ---

#[derive(Clone, Copy, Debug, PartialEq)]
enum TileKind {
    Empty,
    CerealGrass {
        calories: i32,
    },
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    kind: TileKind,
}

trait Consumable {
    fn consume(&mut self) -> i32;
}

impl Consumable for Tile {
    fn consume(&mut self) -> i32 {
        match self.kind {
            TileKind::CerealGrass { calories} => {
                self.kind = TileKind::Empty;
                calories
            },
            TileKind::Empty => 0,
        }
    }
}

// --- Components ---

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component, Debug)]
struct Calories {
    current: i32,
    max: i32,
}

#[derive(Component, Debug, PartialEq)]
enum FsmState {
    Wandering,
    SeekingFood,
    Eating {
        progress: u32,
        max_progress: u32,
    },
}

#[derive(Component, Debug)]
struct Target(Option<Position>);

#[derive(Component)]
struct WantsToMove {
    pub destination: Position,
}

#[derive(Component)]
struct CreatureTag;

#[derive(Component)]
struct TileSprite;

#[derive(Component)]
struct TickText;

// --- Resources ---
// Resources are global, unique data structures

#[derive(Resource)]
struct GameGrid {
    tiles: Vec<Vec<Tile>>,
}

#[derive(Resource, Default)]
struct TickCount(u32);

// --- UX Systems ---

fn toggle_pause_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        match current_state.get() {
            GameState::Running => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Running),
        }
    }
}

fn exit_on_escape_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_events.write(AppExit::Success);
    }
}

// --- Graphics Systems ---

fn setup_system(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    // --- Resource Setup ---
    let mut rng = rand::rng();
    let mut grid_tiles = vec![vec![Tile { kind: TileKind::Empty }; GRID_WIDTH]; GRID_HEIGHT];
    for _ in 0..STARTING_GRASS_COUNT {
        let x = rng.random_range(0..GRID_WIDTH);
        let y = rng.random_range(0..GRID_HEIGHT);
        grid_tiles[y][x].kind = TileKind::CerealGrass { calories: 20 };
    }

    commands.insert_resource(GameGrid { tiles: grid_tiles });
    commands.insert_resource(TickCount::default());

    // --- Spawning Initial Entities ---
    commands.spawn((
        CreatureTag,
        Position { x: 10, y: 10 },
        Calories { current: 100, max: 100 },
        FsmState::Wandering,
        Target(None),
    ));
    commands.spawn((
        CreatureTag,
        Position { x: 15, y: 12 },
        Calories { current: 60, max: 100 },
        FsmState::Wandering,
        Target(None),
    ));
}

fn setup_visualization_system(mut commands: Commands, grid: Res<GameGrid>) {
    // --- Draw the Grid ---
    // We spawn a sprite for each tile only once
    for (y, row) in grid.tiles.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let color = match tile.kind {
                TileKind::Empty => {
                    if (x + y) % 2 == 0 {
                        Color::srgb(0.4, 0.4, 0.4)
                    } else {
                        Color::srgb(0.5, 0.5, 0.5)
                    }
                }
                TileKind::CerealGrass { .. } => Color::srgb(0.2, 0.8, 0.2),
            };

            commands.spawn((
                TileSprite,
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
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
    
    // --- Draw the UI Text ---
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
}

// System to add sprites to newly created creatures
fn spawn_creature_visuals_system(
    mut commands: Commands,
    query: Query<(Entity, &Position), (With<CreatureTag>, Added<Position>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, pos) in query.iter() {
        commands.entity(entity).insert(
            Sprite {
                color: Color::srgb(1.0, 1.0, 0.0), // Default color
                custom_size: Some(Vec2::new(TILE_SIZE * 0.9, TILE_SIZE * 0.9)),
                image: asset_server.load("sprites/human.png"),
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
fn update_creature_position_system(
    mut query: Query<(&mut Transform, &Position), With<CreatureTag>>,
) {
    for (mut transform, pos) in query.iter_mut() {
        transform.translation.x = pos.x as f32 * TILE_SIZE - (GRID_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
        transform.translation.y = pos.y as f32 * TILE_SIZE - (GRID_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    }
}

// System to update creature color based on health
fn update_creature_color_system(mut query: Query<(&mut Sprite, &Calories), With<CreatureTag>>) {
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
fn update_tick_text_system(
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
fn update_tile_visuals_system(
    grid: Res<GameGrid>,
    mut query: Query<(&mut Sprite, &Position), With<TileSprite>>,
) {
    if grid.is_changed() {
        for (mut sprite, pos) in query.iter_mut() {
            let tile = &grid.tiles[pos.y as usize][pos.x as usize];
            sprite.color = match tile.kind {
                TileKind::Empty => {
                    if (pos.x + pos.y) % 2 == 0 {
                        Color::srgb(0.4, 0.4, 0.4)
                    } else {
                        Color::srgb(0.5, 0.5, 0.5)
                    }
                }
                TileKind::CerealGrass { .. } => Color::srgb(0.2, 0.8, 0.2),
            };
        }
    }
}

// --- Game Logic Systems ---

fn fsm_decision_system(
    mut query: Query<(&mut FsmState, &Calories, &mut Target, &Position), With<CreatureTag>>,
    grid: Res<GameGrid>,
) {
    for (mut fsm, calories, mut target, pos) in query.iter_mut() {
        let is_hungry = calories.current < (calories.max as f32 / 2.0) as i32;

        if is_hungry && *fsm == FsmState::Wandering {
            if let Some(food_pos) = find_closest_food(&grid, pos) {
                *fsm = FsmState::SeekingFood;
                target.0 = Some(food_pos);
            }
        } else if !is_hungry && *fsm != FsmState::Wandering {
             // If not hungry anymore, go back to wandering
            if let FsmState::SeekingFood = *fsm {
                *fsm = FsmState::Wandering;
                target.0 = None;
            }
        }
    }
}

fn wandering_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Position, &FsmState), With<CreatureTag>>,
) {
    let mut rng = rand::rng();
    for (entity, pos, fsm) in query.iter_mut() {
        if *fsm == FsmState::Wandering {
            let mut new_pos = *pos;
            match rng.random_range(0..5) {
                0 => new_pos.y = (new_pos.y - 1).max(0),
                1 => new_pos.y = (new_pos.y + 1).min(GRID_HEIGHT as i32 - 1),
                2 => new_pos.x = (new_pos.x - 1).max(0),
                3 => new_pos.x = (new_pos.x + 1).min(GRID_WIDTH as i32 - 1),
                _ => {} // Stay put
            }
            // Instead of changing Position directly, we add a component to signal intent.
            if new_pos != *pos {
                commands.entity(entity).insert(WantsToMove { destination: new_pos });
            }
        }
    }
}

fn seeking_food_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Position, &mut FsmState, &mut Target), With<CreatureTag>>,
    grid: Res<GameGrid>,
) {
    for (entity, pos, mut fsm, mut target) in query.iter_mut() {
        if *fsm == FsmState::SeekingFood {
            if let Some(target_pos) = target.0 {
                // Have we arrived?
                if *pos == target_pos {
                    let tile = &grid.tiles[target_pos.y as usize][target_pos.x as usize];
                    if let TileKind::CerealGrass { .. } = tile.kind {
                        *fsm = FsmState::Eating { progress: 0, max_progress: 3 };
                    } else {
                        // Food is gone, go back to wandering.
                        *fsm = FsmState::Wandering;
                        target.0 = None;
                    }
                    continue; // Done with this entity for this tick
                }

                // Move towards the target
                let mut new_pos = *pos;
                let dx = target_pos.x - pos.x;
                let dy = target_pos.y - pos.y;

                if dx.abs() > dy.abs() {
                    new_pos.x += dx.signum();
                } else {
                    new_pos.y += dy.signum();
                }
                commands.entity(entity).insert(WantsToMove { destination: new_pos });
            } else {
                // No target? Go back to wandering.
                *fsm = FsmState::Wandering;
            }
        }
    }
}

fn eating_system(
    mut query: Query<(&Position, &mut FsmState, &mut Calories), With<CreatureTag>>,
    mut grid: ResMut<GameGrid>,
) {
    for (pos, mut fsm, mut calories) in query.iter_mut() {
        if let FsmState::Eating { progress, max_progress } = *fsm {
            let new_progress = progress + 1;
            calories.current -= WORK_COST;

            if new_progress >= max_progress {
                let current_tile = &mut grid.tiles[pos.y as usize][pos.x as usize];
                let calories_gained = current_tile.consume();
                calories.current += calories_gained;
                
                // Finished eating, go back to wandering
                *fsm = FsmState::Wandering;
            } else {
                *fsm = FsmState::Eating { progress: new_progress, max_progress };
            }
        }
    }
}

fn movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Position, &WantsToMove, &mut Calories)>,
) {
    for (entity, mut pos, wants_to_move, mut calories) in query.iter_mut() {
        if *pos != wants_to_move.destination {
            *pos = wants_to_move.destination;
            calories.current -= MOVE_COST;
        }
        // The intent has been handled, so we remove the component.
        commands.entity(entity).remove::<WantsToMove>();
    }
}

fn calorie_burn_system(mut query: Query<&mut Calories, With<CreatureTag>>) {
    for mut calories in query.iter_mut() {
        calories.current -= LIVE_COST;
    }
}

fn death_system(mut commands: Commands, query: Query<(Entity, &Calories)>) {
    for (entity, calories) in query.iter() {
        if calories.current <= 0 {
            commands.entity(entity).despawn();
        }
    }
}

fn tick_counter_system(mut tick_count: ResMut<TickCount>) {
    tick_count.0 += 1;
}

// --- Helper Functions ---
fn find_closest_food(grid: &Res<GameGrid>, position: &Position) -> Option<Position> {
    let mut closest_food: Option<(Position, i32)> = None;

    for (y, row) in grid.tiles.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            if let TileKind::CerealGrass { .. } = tile.kind {
                let food_pos = Position { x: x as i32, y: y as i32 };
                let dist = (position.x - food_pos.x).abs() + (position.y - food_pos.y).abs(); // Manhattan distance

                if let Some((_, min_dist)) = closest_food {
                    if dist < min_dist {
                        closest_food = Some((food_pos, dist));
                    }
                } else {
                    closest_food = Some((food_pos, dist));
                }
            }
        }
    }
    closest_food.map(|(pos, _)| pos)
}
