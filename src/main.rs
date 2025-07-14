use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color};
use ggez::event::{self, EventHandler};

const GRID_WIDTH: usize = 40;
const GRID_HEIGHT: usize = 30;
const TILE_SIZE: f32 = 20.0;

// A struct to represent a single tile in our grid.
// `#[derive(...)]` gives our struct some default behaviors.
// `Copy` and `Clone` allow us to easily duplicate tiles.
#[derive(Clone, Copy, Debug)]
struct Tile {
    // For now, a tile doesn't need any data, but we'll add things like
    // color or type later.
}

// This struct will hold all our game's data.
// For now, it's empty.
struct GameState {
    grid: Vec<Vec<Tile>>,
}

// This is our constructor. It's a convention to name it `new`.
impl GameState {
    fn new() -> GameResult<Self> {
        // Create the grid, filling it with default tiles.
        // `vec!` is a macro that creates a Vec.
        // `[Tile {}; GRID_WIDTH]` creates an array of `GRID_WIDTH` tiles.
        // The outer `vec!` does this `GRID_HEIGHT` times to create all the rows.
        let grid = vec![vec![Tile {}; GRID_WIDTH]; GRID_HEIGHT];

        Ok(GameState { grid })
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
            for (x, _tile) in row.iter().enumerate() {
                // Determine the color of the square. We'll make a checkerboard.
                let color = if (x + y) % 2 == 0 {
                    Color::from([0.4, 0.4, 0.4, 1.0]) // Dark grey
                } else {
                    Color::from([0.5, 0.5, 0.5, 1.0]) // Light grey
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

        // 3. Present the canvas to the screen.
        canvas.finish(ctx)?;

        Ok(())
    }
}

// This is the entry point of our program.
pub fn main() -> GameResult {
    // 1. Create a context and a window configuration.
    let (mut ctx, event_loop) = ContextBuilder::new("guns_germs_steel", "Oleksiy")
        .window_setup(ggez::conf::WindowSetup::default().title("Guns, Germs, and Steel!"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0))
        .build()?;

    // 2. Create an instance of our game state.
    let state = GameState::new()?;

    // 3. Start the game loop.
    event::run(ctx, event_loop, state)
}