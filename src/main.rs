use bevy::prelude::*;

mod resources;
mod systems;
mod components;
mod constants;

use resources::{
    game_state::GameState,
};
use systems::{
    ux::*,
    setup::*,
    graphics::*,
    gameplay::*,
};
use constants::*;

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
