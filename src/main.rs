use bevy::prelude::*;
use bevy::input::common_conditions::input_just_pressed;

mod resources;
mod systems;
mod components;
mod constants;

use components::components::FoodTargetInvalidated;

use resources::{
    game_state::GameState,
    camera::{CameraZoom, CameraPosition},
    ui_elements::{BandCenterVisualizationEnabled, LeftPanelState},
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
        .init_resource::<BandCenterVisualizationEnabled>()
        .init_resource::<LeftPanelState>()
        .add_event::<FoodTargetInvalidated>()
        .add_systems(
            Startup, 
            (
                setup_system,
                setup_visualization_system,
                spawn_ui,
            ).chain(),
        )
        .add_systems(
            FixedUpdate, // System run every tick
            (
                update_band_center_system,
                check_manual_band_return_system,
                // Intent-Driven Systems
                goal_selection_system,      // Brain: assigns intents (WantsTo*)
                idle_goal_selection_system,   // Convert WantsToIdle to actions
                find_food_system,          // Convert WantsToEat to actions  
                pathfinding_system,        // Convert ActionTravelTo to ActivePath
                return_to_band_system,      // Convert WantsToReturnToBand to ActionTravelTo
                perform_movement_system,    // Execute movement along ActivePath
                perform_eat_system,        // Execute eating actions
                food_target_notification_system, // Notify creatures when their targets become unavailable
                handle_food_target_invalidated_system, // Handle food target invalidation events
                procreation_system,        // Execute procreation actions
                check_if_returned_to_band_system, // Remove OutsideBandRadius if returned to band
                // Core systems
                pregnancy_system,
                calorie_burn_system,
                death_system,
                //plant_propagation_system, // TODO: Remove when not needed
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
                    band_center_toggle_system,
                    camera_zoom_system,
                    camera_pan_system,
                    spawn_creature_visuals_system,
                    spawn_plant_visuals_system,
                    update_creature_color_system,
                    update_creature_position_visuals_system,
                    update_population_text_system,
                    update_selected_panel_system,
                    path_visualization_system,
                    cleanup_path_visualization_system,
                    band_center_visualization_system,
                    update_tick_text_system,
                    cursor_click_system.run_if(input_just_pressed(MouseButton::Left)),
                    clear_selection_on_escape_system,
                ),
            ).chain(),
        )
        .insert_resource(Time::<Fixed>::from_hz(TICK_RATE_HZ))
        .run();
}
