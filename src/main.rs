use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color};
use ggez::event::{self, EventHandler};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::mint::{Point2};
use rand::Rng;
use bevy_ecs::prelude::*;

const GRID_WIDTH: usize = 40;
const GRID_HEIGHT: usize = 30;
const TILE_SIZE: f32 = 20.0;
const STARTING_GRASS_COUNT: i32 = 80;
const MOVE_COST: i32 = 1;
const LIVE_COST: i32 = 1;
const WORK_COST: i32 = 1;

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

// --- Resources ---
// Resources are global, unique data structures

#[derive(Resource)]
struct GameGrid {
    tiles: Vec<Vec<Tile>>,
}

#[derive(Resource, Default)]
struct TickCount(u32);

// --- Systems ---

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

// --- Game State ---

struct GameState {
    world: World,
    schedule: Schedule,
}

impl GameState {
    fn new() -> GameResult<Self> {
        // --- World and Resource Setup ---
        let mut world = World::new();

        let mut rng = rand::rng();
        let mut grid_tiles = vec![vec![Tile { kind: TileKind::Empty }; GRID_WIDTH]; GRID_HEIGHT];
        for _ in 0..STARTING_GRASS_COUNT {
            let x = rng.random_range(0..GRID_WIDTH);
            let y = rng.random_range(0..GRID_HEIGHT);
            grid_tiles[y][x].kind = TileKind::CerealGrass { calories: 20 };
        }
        world.insert_resource(GameGrid { tiles: grid_tiles });
        world.insert_resource(TickCount::default());

        // --- Spawning Initial Entities ---
        world.spawn((
            CreatureTag,
            Position { x: 10, y: 10 },
            Calories { current: 100, max: 100 },
            FsmState::Wandering,
            Target(None),
        ));
        world.spawn((
            CreatureTag,
            Position { x: 15, y: 12 },
            Calories { current: 60, max: 100 },
            FsmState::Wandering,
            Target(None),
        ));

        // --- Schedule Setup ---
        // The order of systems matters!
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                // Logic systems first, to generate "wants"
                fsm_decision_system,
                wandering_system,
                seeking_food_system,
                eating_system,
                // Apply the "wants"
                movement_system,
                // Apply costs and check for death
                calorie_burn_system,
                death_system,
                // Finally, update tick counter
                tick_counter_system,
            )
            .chain(), // .chain() ensures they run in this specific order
        );

        Ok(GameState { world, schedule })
    }
}

// ggez will call these methods in a loop.
impl EventHandler for GameState {
    // The `update` method is called on every frame before drawing.
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(keycode) = input.keycode {
            if keycode == KeyCode::Space {
                self.schedule.run(&mut self.world);
            }
        }
        Ok(())
    }

    // The `draw` method is called on every frame after updating.
    // It's where you'll draw everything to the screen.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // 1. Create a canvas to draw on.
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));

        // 2. Drawing logic
        // Iterate over the grid with both index and value.
        let grid = self.world.resource::<GameGrid>();
        for (y, row) in grid.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let color = match tile.kind {
                    TileKind::Empty => {
                        // Checkerboard pattern
                        if (x + y) % 2 == 0 {
                            Color::from([0.4, 0.4, 0.4, 1.0]) // Dark grey
                        } else {
                            Color::from([0.5, 0.5, 0.5, 1.0]) // Light grey
                        }
                    }
                    TileKind::CerealGrass { .. } => Color::from([0.2, 0.8, 0.2, 1.0]),
                };

                // Create a rectangle mesh for the tile.
                let rect = graphics::Rect::new(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                );

                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::new()
                        .dest(rect.point())
                        .scale(rect.size())
                        .color(color),
                );
            }
        }

        // 3. Draw the creatures
        let mut creature_query = self.world.query::<(&Position, &Calories)>();
        for (position, calories) in creature_query.iter(&self.world) {
            let center_x = (position.x as f32 * TILE_SIZE) + TILE_SIZE / 2.0;
            let center_y = (position.y as f32 * TILE_SIZE) + TILE_SIZE / 2.0;
            let color = if calories.current >= calories.max {
                Color::from([0.0, 1.0, 0.0, 1.0]) // 100%+
            } else if calories.current >= (calories.max as f32 / 2.0) as i32 {
                Color::from([1.0, 1.0, 0.0, 1.0]) // 50%-99%
            } else if calories.current >= (calories.max as f32 / 4.0) as i32 {
                Color::from([1.0, 0.5, 0.0, 1.0]) // 25%-49%
            } else {
                Color::from([1.0, 0.0, 0.0, 1.0]) // 0%-24%
            };

            let circle = graphics::Mesh::new_circle(
                &mut ctx.gfx,
                graphics::DrawMode::Fill(graphics::FillOptions::default()),
                Point2 { x: 0.0, y: 0.0 }, // Create at origin
                TILE_SIZE / 2.2,
                0.1,
                color,
            )?;

            canvas.draw(
                &circle,
                Point2 { x: center_x, y: center_y },
            );
        }

        // 4. Draw the tick count
        let tick_count = self.world.resource::<TickCount>();
        let text = graphics::Text::new(format!("Tick: {}", tick_count.0));
        canvas.draw(
            &text,
            Point2 { x: 10.0, y: 10.0 },
        );

        // 5. Present the canvas to the screen.
        canvas.finish(ctx)?;
        Ok(())
    }
}

// This is the entry point of our program.
pub fn main() -> GameResult {
    // 1. Create a context and a window configuration.
    let (ctx, event_loop) = ContextBuilder::new("guns_germs_steel", "Oleksiy")
        .window_setup(ggez::conf::WindowSetup::default().title("Guns, Germs, and Steel!"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(
            GRID_WIDTH as f32 * TILE_SIZE,
            GRID_HEIGHT as f32 * TILE_SIZE,
        ))
        .build()?;

    // 2. Create an instance of our game state.
    let state = GameState::new()?;

    // 3. Start the game loop.
    event::run(ctx, event_loop, state)
}