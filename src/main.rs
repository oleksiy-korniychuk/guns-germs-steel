use bevy::prelude::*;
use bevy::input::common_conditions::input_just_pressed;

mod resources;
mod systems;
mod components;
mod constants;

use resources::{
    game_state::GameState,
    camera_zoom::CameraZoom,
    camera_position::CameraPosition,
};
use systems::{
    ux::*,
    setup::*,
    graphics::*,
    gameplay::*,
    creature::*,
    input::*,
};
use constants::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Guns, Germs, and Steel!".into(),
                resolution: (
                    DEFAULT_WINDOW_WIDTH,
                    DEFAULT_WINDOW_HEIGHT,
                ).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<CameraZoom>()
        .init_resource::<CameraPosition>()
        .add_systems(
            Startup, 
            (
                setup_system,
                setup_visualization_system,
            ).chain(),
        )
        .add_systems(
            FixedUpdate, // System run every tick
            (
                update_band_center_system,
                // Intent-Driven Systems
                goal_selection_system,      // Brain: assigns intents (WantsTo*)
                idle_goal_selection_system,   // Convert WantsToIdle to actions
                find_food_system,          // Convert WantsToEat to actions  
                pathfinding_system,        // Convert ActionTravelTo to ActivePath
                return_to_band_system,      // Convert WantsToReturnToBand to ActionTravelTo
                perform_movement_system,    // Execute movement along ActivePath
                perform_eat_system,        // Execute eating actions
                procreation_system,        // Execute procreation actions
                check_if_returned_to_band_system, // Remove OutsideBandRadius if returned to band
                // Core systems
                pregnancy_system,
                calorie_burn_system,
                death_system,
                //plant_propogation_system, // TODO: Remove when not needed
                population_counter_system,
                tick_counter_system,
            ).chain().run_if(in_state(GameState::Running)),
        )
        .add_systems(
            Update, // System run every frame
            (
                spatial_grid_system,
                (
                    toggle_pause_system,
                    exit_on_escape_system,
                    camera_zoom_system,
                    camera_pan_system,
                    spawn_creature_visuals_system,
                    spawn_plant_visuals_system,
                    update_creature_color_system,
                    update_creature_position_visuals_system,
                    update_population_text_system,
                    path_visualization_system,
                    cleanup_path_visualization_system,
                    update_tick_text_system,
                    cursor_click_system.run_if(input_just_pressed(MouseButton::Left))
                ),
            ).chain(),
        )
        .insert_resource(Time::<Fixed>::from_hz(TICK_RATE_HZ))
        .run();
}
