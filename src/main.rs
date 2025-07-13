use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color};
use ggez::event::{self, EventHandler};

// This struct will hold all our game's data.
// For now, it's empty.
struct GameState {}

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

        // 2. We're not drawing anything yet, just clearing the screen with a dark color.

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
    let state = GameState {};

    // 3. Start the game loop.
    event::run(ctx, event_loop, state)
}