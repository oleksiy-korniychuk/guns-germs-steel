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
const MOVE_COST: i32 = 1;
const LIVE_COST: i32 = 1;
const WORK_COST: i32 = 1;

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

#[derive(Clone, Copy, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Wandering,
    SeekingFood,
    Eating { 
        progress: u32, 
        max_progress: u32 
    },
}

#[derive(Debug)]
struct Creature {
    pub position: Position,
    pub state: State,
    pub target: Option<Position>,
    pub calories: i32,
    pub calories_max: i32,
}

trait Consumable {
    fn consume(&mut self) -> i32;
}

struct GameState {
    grid: Vec<Vec<Tile>>,
    creatures: Vec<Creature>,
    tick_count: u32,
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

        Ok(GameState { grid, creatures, tick_count: 0 })
    }

    fn tick(&mut self) {
        let mut i = 0;
        while i < self.creatures.len() {
            // Update the creature's state and behavior
            let creature_state = self.creatures[i].state;
            match creature_state {
                State::Wandering => self.handle_wandering_at_index(i),
                State::SeekingFood => self.handle_seeking_food_at_index(i),
                State::Eating { .. } => self.handle_eating_at_index(i),
            }
            
            // Base calorie drain
            self.creatures[i].calories -= LIVE_COST;
            
            // Remove if dead
            if self.creatures[i].calories <= 0 {
                self.creatures.remove(i);
            } else {
                i += 1;
            }
        }

        self.tick_count += 1;
    }

    // --- Helper Functions ---

    fn handle_wandering_at_index(&mut self, index: usize) {
        // If the creature gets hungry, it starts seeking food.
        if self.creatures[index].calories < (self.creatures[index].calories_max as f32 / 2.0) as i32 {
            if let Some(food_pos) = self.find_closest_food(self.creatures[index].position) {
                self.creatures[index].state = State::SeekingFood;
                self.creatures[index].target = Some(food_pos);
                return; // State changed, so we are done for this tick.
            }
        }

        // Otherwise, move randomly
        let mut rng = rand::rng();
        let mut new_pos = self.creatures[index].position;
        match rng.random_range(0..5) {
            0 => new_pos.y = (new_pos.y - 1).max(0), // Up
            1 => new_pos.y = (new_pos.y + 1).min(GRID_HEIGHT as i32 - 1), // Down
            2 => new_pos.x = (new_pos.x - 1).max(0), // Left
            3 => new_pos.x = (new_pos.x + 1).min(GRID_WIDTH as i32 - 1), // Right
            _ => new_pos.x = new_pos.x, // Stay in place
        }
        self.creatures[index].position = new_pos;
        self.creatures[index].calories -= MOVE_COST; // Moving costs extra
    }

    fn find_closest_food(&self, position: Position) -> Option<Position> {
        let mut closest_food: Option<(Position, i32)> = None;

        for (y, row) in self.grid.iter().enumerate() {
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

    fn handle_seeking_food_at_index(&mut self, index: usize) {
        let target_pos = match self.creatures[index].target {
            Some(pos) => pos,
            None => { // Should not happen, but as a fallback, go back to wandering.
                self.creatures[index].state = State::Wandering;
                return;
            }
        };
    
        // Have we arrived?
        if self.creatures[index].position.x == target_pos.x && self.creatures[index].position.y == target_pos.y {
            // Check if the food is still there
            let tile = &self.grid[target_pos.y as usize][target_pos.x as usize];
            if let TileKind::CerealGrass { .. } = tile.kind {
                self.creatures[index].state = State::Eating { progress: 0, max_progress: 3 };
            } else {
                // The food was eaten by someone else! Go back to wandering.
                self.creatures[index].state = State::Wandering;
                self.creatures[index].target = None;
            }
            return;
        }
    
        // Move towards the target (simple pathfinding)
        let mut new_pos = self.creatures[index].position;
        let dx = target_pos.x - self.creatures[index].position.x;
        let dy = target_pos.y - self.creatures[index].position.y;
    
        if dx.abs() > dy.abs() {
            new_pos.x += dx.signum();
        } else {
            new_pos.y += dy.signum();
        }
        
        // Boundary checks
        if new_pos.x >= 0 && new_pos.x < GRID_WIDTH as i32 &&
           new_pos.y >= 0 && new_pos.y < GRID_HEIGHT as i32 {
            self.creatures[index].position = new_pos;
            self.creatures[index].calories -= MOVE_COST; // Moving costs extra
        }
    }

    fn handle_eating_at_index(&mut self, index: usize) {
        let (progress, max_progress) = if let State::Eating { progress, max_progress } = self.creatures[index].state {
            (progress, max_progress)
        } else {
            return;
        };

        let new_progress = progress + 1;
        self.creatures[index].calories -= WORK_COST; // Gathering is hard work

        if new_progress >= max_progress {
            let position = self.creatures[index].position;
            let current_tile = &mut self.grid[position.y as usize][position.x as usize];
            let calories_gained = current_tile.consume();
            self.creatures[index].calories += calories_gained;
            
            // After eating, go back to wandering
            self.creatures[index].state = State::Wandering;
            self.creatures[index].target = None;
        } else {
            self.creatures[index].state = State::Eating { progress: new_progress, max_progress };
        }
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
        for (y, row) in self.grid.iter().enumerate() {
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

        // 4. Draw the tick count
        let text = graphics::Text::new(format!("Tick: {}", self.tick_count));
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
        state: State::Wandering,
        target: None,
    });

    state.creatures.push(Creature {
        position: Position { x: 15, y: 12 },
        calories: 60,
        calories_max: 100,
        state: State::Wandering,
        target: None,
    });

    // 3. Start the game loop.
    event::run(ctx, event_loop, state)
}