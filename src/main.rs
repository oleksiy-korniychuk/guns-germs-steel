use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color};
use ggez::event::{self, EventHandler};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::mint::{Point2};
use rand::Rng;

const GRID_WIDTH: usize = 40;
const GRID_HEIGHT: usize = 30;
const TILE_SIZE: f32 = 20.0;
const STARTING_GRASS_COUNT: i32 = 80;

#[derive(Clone, Copy, Debug, PartialEq)]
enum TileKind {
    Empty,
    CerealGrass {
        calories: i32,
    },
}

// A struct to represent a single tile in our grid.
// `#[derive(...)]` gives our struct some default behaviors.
// `Copy` and `Clone` allow us to easily duplicate tiles.
#[derive(Clone, Copy, Debug)]
struct Tile {
    kind: TileKind,
}

#[derive(Clone, Copy, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Action {
    Idle,
    Gathering {
        progress: u32,
        max_progress: u32,
    }
}

#[derive(Debug)]
struct Creature {
    pub position: Position,
    pub calories: i32,
    pub calories_max: i32,
    pub action: Action,
}

trait Consumable {
    fn consume(&mut self) -> i32;
}

// This struct will hold all our game's data.
// For now, it's empty.
struct GameState {
    grid: Vec<Vec<Tile>>,
    creatures: Vec<Creature>,
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

// This is our constructor. It's a convention to name it `new`.
impl GameState {
    fn new() -> GameResult<Self> {
        let mut rng = rand::rng();

        let mut grid = vec![vec![Tile { kind: TileKind::Empty }; GRID_WIDTH]; GRID_HEIGHT];

        // Seed the grid with some cereal grass
        for _ in 0..STARTING_GRASS_COUNT {
            let x = rng.random_range(0..GRID_WIDTH);
            let y = rng.random_range(0..GRID_HEIGHT);
            grid[y][x].kind = TileKind::CerealGrass { calories: 20 };
        }

        let creatures = Vec::new();

        Ok(GameState { grid, creatures })
    }

    fn tick(&mut self) {
        let mut rng = rand::rng();

        // We use `retain_mut` to iterate and modify our creatures.
        // It allows us to safely remove creatures from the vector while iterating.
        self.creatures.retain_mut(|creature| {
            // --- Base Calorie Drain ---
            // It costs energy just to exist.
            let mut cost = 1;

            // --- Action Logic ---
            // Use a `match` statement to perform logic based on the creature's current action.
            match creature.action {
                Action::Idle => {
                    // When Idle, the creature first checks its surroundings.
                    let current_tile = &mut self.grid[creature.position.y as usize][creature.position.x as usize];
                    
                    // `if let` is a handy way to check if an enum is a specific variant.
                    // Here, we check if the tile's kind is CerealGrass.
                    if let TileKind::CerealGrass { .. } = current_tile.kind {
                        // If it's on a grass tile, it starts gathering.
                        creature.action = Action::Gathering { progress: 0, max_progress: 3 }; // Takes 3 ticks
                    } else {
                        // If the tile is empty, the creature moves.
                        let direction = rng.gen_range(0..4); // 0: Up, 1: Down, 2: Left, 3: Right
                        let mut new_pos = creature.position;
                        match direction {
                            0 => new_pos.y -= 1,
                            1 => new_pos.y += 1,
                            2 => new_pos.x -= 1,
                            _ => new_pos.x += 1,
                        }

                        // Boundary checks
                        if new_pos.x >= 0 && new_pos.x < GRID_WIDTH as i32 &&
                           new_pos.y >= 0 && new_pos.y < GRID_HEIGHT as i32 {
                            creature.position = new_pos;
                            cost += 1; // Moving costs extra.
                        }
                    }
                }
                Action::Gathering { ref mut progress, max_progress } => {
                    // If gathering, increment progress.
                    *progress += 1;
                    cost += 1; // Gathering is hard work and costs energy.

                    if *progress >= max_progress {
                        // Finished gathering. Consume the resource.
                        let current_tile = &mut self.grid[creature.position.y as usize][creature.position.x as usize];
                        
                        // We call the `consume` method from our `Consumable` trait!
                        let calories_gained = current_tile.consume();
                        creature.calories += calories_gained;

                        // Clamp calories to the maximum.
                        if creature.calories > creature.calories_max {
                            creature.calories = creature.calories_max;
                        }
                        
                        // After eating, the creature is Idle again.
                        creature.action = Action::Idle;
                    }
                }
            }

            // --- Apply Costs & Check for Survival ---
            creature.calories -= cost;

            // --- Survival Check ---
            // The closure returns `true` to keep the creature, `false` to remove it.
            creature.calories > 0
        });

    }
}

// This is where we implement the logic for our game.
// ggez will call these methods in a loop.
impl EventHandler for GameState {
    // The `update` method is called on every frame before drawing.
    // It's where you'll put your game logic, like moving characters.
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // We don't have any logic yet, so we just return Ok.
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(keycode) = input.keycode {
            if keycode == KeyCode::Space {
                self.tick();
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
        // `enumerate()` gives us the index (y) of each row.
        for (y, row) in self.grid.iter().enumerate() {
            // `iter()` lets us look at the items in the row without taking ownership.
            for (x, tile) in row.iter().enumerate() {
                // Determine the color of the square. We'll make a checkerboard.
                let color = match tile.kind {
                    TileKind::Empty => {
                        if (x + y) % 2 == 0 {
                            Color::from([0.4, 0.4, 0.4, 1.0]) // Dark grey
                        } else {
                            Color::from([0.5, 0.5, 0.5, 1.0]) // Light grey
                        }
                    }
                    // CerealGrass tiles are now green!
                    TileKind::CerealGrass { .. } => Color::from([0.2, 0.8, 0.2, 1.0]),
                };

                // Create a rectangle mesh for the tile.
                let rect = graphics::Rect::new(
                    x as f32 * TILE_SIZE, // The `as f32` is a type cast
                    y as f32 * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                );

                // Draw the rectangle to the canvas.
                canvas.draw(
                    &graphics::Quad, // We are drawing a simple rectangle
                    graphics::DrawParam::new()
                        .dest(rect.point())
                        .scale(rect.size())
                        .color(color),
                );
            }
        }

        // 3. Draw the creatures
        for creature in self.creatures.iter() {
            let center_x = (creature.position.x as f32 * TILE_SIZE) + TILE_SIZE / 2.0;
            let center_y = (creature.position.y as f32 * TILE_SIZE) + TILE_SIZE / 2.0;
            let color = if creature.calories >= creature.calories_max {
                Color::from([0.0, 1.0, 0.0, 1.0]) // 100%+
            } else if creature.calories >= (creature.calories_max as f32 / 2.0) as i32 {
                Color::from([1.0, 1.0, 0.0, 1.0]) // 50%-99%
            } else if creature.calories >= (creature.calories_max as f32 / 4.0) as i32 {
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

        // 4. Present the canvas to the screen.
        canvas.finish(ctx)?;
        Ok(())
    }
}

// This is the entry point of our program.
pub fn main() -> GameResult {
    // 1. Create a context and a window configuration.
    let (mut ctx, event_loop) = ContextBuilder::new("guns_germs_steel", "Oleksiy")
        .window_setup(ggez::conf::WindowSetup::default().title("Guns, Germs, and Steel!"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(GRID_WIDTH as f32 * TILE_SIZE, GRID_HEIGHT as f32 * TILE_SIZE))
        .build()?;

    // 2. Create an instance of our game state.
    let mut state = GameState::new()?;

    state.creatures.push(Creature {
        position: Position { x: 10, y: 10 },
        calories: 100,
        calories_max: 100,
        action: Action::Idle,
    });

    state.creatures.push(Creature {
        position: Position { x: 15, y: 12 },
        calories: 60,
        calories_max: 100,
        action: Action::Idle,
    });

    // 3. Start the game loop.
    event::run(ctx, event_loop, state)
}