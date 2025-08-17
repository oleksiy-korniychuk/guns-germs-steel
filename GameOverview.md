## Guns, Germs, and Steel -- Game Overview

This document is a concise overview to help a new developer quickly understand the design, architecture, and game flow of this Bevy-based simulation. It includes embedded Mermaid diagrams where useful.

### 1) Project overview
- **Engine**: Bevy 0.16 (ECS, 2D sprites, input, states, fixed timestep)
- **Concept**: Top‑down grid world with creatures that eat wheat, reproduce, move via A*, and die if calories deplete
- **Loop**: Simulation logic runs on FixedUpdate ticks; visuals and input run per‑frame on Update
- **Entry**: `src/main.rs`; constants in `src/constants.rs`

### 2) Architecture and project structure
- `src/main.rs`: App setup, schedules, system registration
- `src/components/`: ECS components and markers
- `src/resources/`: Global resources (grid, state, counts, camera, seed)
- `src/systems/`: Systems grouped by domain (setup, gameplay, creature AI, graphics, input, UX)
- `assets/sprites/`: Unit and tile overlay images

```mermaid
flowchart LR
  main[main.rs] --> systems
  main --> components
  main --> resources
  systems -->|uses| components
  systems -->|reads/writes| resources
  setup[systems/setup.rs] -->|creates| resources
  setup -->|spawns| components
  creature[systems/creature.rs] -->|AI/Path/Actions| components
  gameplay[systems/gameplay.rs] -->|counters, spatial grid| resources
  graphics[systems/graphics.rs] -->|sprites/UI| components
  input[systems/input.rs] -->|camera, clicks| resources
  resources -.->|GameGrid, SpatialGrid, State| components
```

### 3) Scheduling and game state
- Schedules
  - Startup: world generation, camera/UI init, initial spawns
  - FixedUpdate (when `GameState::Running`): simulation tick chain (AI, pathfinding, actions, metabolism, pregnancy, death, counters)
  - Update: per‑frame systems (spatial grid, input, camera, visuals, UI text, path viz)
- State: `GameState` (`Running`/`Paused`), toggled with Space

```mermaid
flowchart TD
    A[Startup] --> B[setup_system]
    B --> C[setup_visualization_system]
    C -->|Init resources + entities| S[GameState::Running]

    subgraph FixedUpdate [FixedUpdate — Runs when Running]
      direction TB
      F1[update_band_center_system]
      F2[goal_selection_system]
      F3[idle_goal_selection_system]
      F4[find_food_system]
      F5[pathfinding_system]
      F6[return_to_band_system]
      F7[perform_movement_system]
      F8[perform_eat_system]
      F9[procreation_system]
      F10[check_if_returned_to_band_system]
      F11[pregnancy_system]
      F12[calorie_burn_system]
      F13[death_system]
      F14[population_counter_system]
      F15[tick_counter_system]
      F1 --> F2 --> F3 --> F4 --> F5 --> F6 --> F7 --> F8 --> F9 --> F10 --> F11 --> F12 --> F13 --> F14 --> F15
    end

    subgraph Update [Update — Every frame]
      direction TB
      U1[spatial_grid_system]
      U2[toggle_pause_system]
      U3[exit_on_escape_system]
      U4[camera_zoom_system]
      U5[camera_pan_system]
      U6[spawn_creature_visuals_system]
      U7[spawn_plant_visuals_system]
      U8[update_creature_color_system]
      U9[update_creature_position_visuals_system]
      U10[path_visualization_system]
      U11[cleanup_path_visualization_system]
      U12[update_population_text_system]
      U13[update_tick_text_system]
      U14["cursor_click_system (on LMB)"]
      U1 --> U2 --> U3 --> U4 --> U5 --> U6 --> U7 --> U8 --> U9 --> U10 --> U11 --> U12 --> U13 --> U14
    end

    S --> FixedUpdate
    S --> Update
    Space[Space key] -->|Toggle| S
```

Notes:
- `Time::<Fixed>::from_hz(TICK_RATE_HZ)` defines simulation tick rate (default 2 Hz).
- The spatial grid is rebuilt every frame; AI uses it during FixedUpdate to find nearby food.

### 4) Data model: components and resources
- Components (selected)
  - Position (i32 x/y grid), Calories (current/max), FoodSource
  - Intents: `WantsToEat`, `WantsToIdle`, `WantsToProcreate`, `WantsToReturnToBand`
  - Actions: `ActionTravelTo {destination}`, `ActionEat {target_entity, progress, max_progress}`
  - Movement: `ActivePath { nodes: Vec<Position> }`
  - Status/markers: `CreatureMarker`, `PlantMarker { PlantType }`, `Harvestable`, `Edible`, `Pregnant`, `OutsideBandRadius`, `TileMarker`, `PathVisualizationEnabled`, `PathMarker { creature_entity }`, UI markers (`TickText`, `PopulationText`)
- Resources
  - `GameGrid { tiles: Vec<Vec<Tile>> }` with `Tile { kind, move_cost }`
  - `SpatialGrid(HashMap<Position, Vec<Entity>>)` for quick occupancy lookups
  - `BandCenter(Position)`, `TickCount(u32)`, `PopulationCount(u32)`, `WorldSeed(u32)`
  - Camera: `CameraZoom(f32)`, `CameraPosition(Vec2)`
- Configuration
  - `src/constants.rs` covers grid/window sizes, tick rate, world gen thresholds, movement costs, pregnancy duration, band radius, headband colors

```mermaid
classDiagram
  class Creature {
    +Position
    +Calories
    +CreatureMarker
    +ActivePath?
    +Pregnant?
    +WantsToEat?/WantsToIdle?/WantsToProcreate?/WantsToReturnToBand?
    +ActionTravelTo?/ActionEat?
  }
  class Plant {
    +Position
    +FoodSource
    +PlantMarker
    +Harvestable
    +Edible
  }
  class TileSprite {
    +Position
    +TileMarker
  }
  class Resources {
    +GameGrid
    +SpatialGrid
    +BandCenter
    +TickCount
    +PopulationCount
    +WorldSeed
    +CameraZoom
    +CameraPosition
  }
```

### 5) Gameplay flow (tick)
- Intent selection: If outside band radius → return; else if hungry → eat; else if well‑fed and not pregnant → procreate; else idle
- Intent to action: Idle picks a neighbor tile randomly; Eat finds nearest available plant; Return sets `ActionTravelTo` band center
- Pathfinding: A* over `GameGrid` with costs (water very expensive)
- Action execution: movement consumes MOVE_COST; eating consumes WORK_COST until complete, then grants nutrition and despawns plant
- Metabolism & lifecycle: burn calories each tick; pregnancy progresses and spawns a new creature when done; death on <= 0 calories
- Counters and band: population and tick counters updated; band center is average of creature positions

```mermaid
flowchart LR
  I[Intents] --> A[Actions]
  A --> P[A* Path]
  P --> M[Movement]
  M --> E[Eating]
  E --> C[Calories ±]
  C -->|<=0| D[Death]
  C -->|>threshold| R[Procreate]
  M --> B[Band radius check]
```

### 6) Rendering & UX (frame)
- Sprites: creature base sprite + headband child sprite; plants use wheat sprite; tiles colored per kind
- Positions: world coordinates derived from grid (`TILE_SIZE`, map centered at origin)
- UI text: tick and population updated when resources change
- Optional path visualization: toggled per‑creature via click; markers are ephemeral and cleaned up

### 7) Input & camera
- Input: Space toggles pause; Escape exits; Left click selects a tile and toggles path viz for creatures under cursor (also logs plant info)
- Camera zoom: mouse wheel adjusts `CameraZoom` clamped between `MIN_ZOOM` and a map‑fit max
- Camera pan: WASD moves camera with bounds so the viewport never goes outside the map (unless map is smaller than viewport)

### 8) Build & run
- Requirements: Rust toolchain
- Run: `cargo run` (window size from constants; tick rate via `TICK_RATE_HZ`)

### 9) Extension points and conventions
- Adding systems: register in the appropriate schedule in `main.rs`; maintain chain order for deterministic ticks
- New components/resources: define under `src/components` or `src/resources`; import in systems; prefer small, focused components
- Visual rules: keep grid‑to‑world mapping consistent; layer using Z to ensure sprites render above tiles
- Gameplay: follow intent→action→execution pattern; read/modify calories/movement costs through constants for balancing

```mermaid
flowchart LR
  GameGrid -->|neighbors+costs| Pathfinding
  SpatialGrid -->|nearest edible| FindFood
  BandCenter --> ReturnToBand --> CheckReturned
  CameraZoom --> Zoom
  CameraPosition --> Pan
```

### 11) Quick glossary
- **Intent components**: transient “wants to” markers driving AI selection
- **Action components**: concrete, executable steps produced from intents
- **ActivePath**: queue of `Position` nodes produced by A* consumed by movement
- **Spatial grid**: hash‑based occupancy map for O(1) entity lookups per tile

### 12) Key files to skim first
- `src/main.rs` — schedule map and system order
- `src/components/components.rs` — components, intents, actions, markers
- `src/resources/game_grid.rs` — world data and spatial occupancy
- `src/systems/creature.rs` — AI flow, pathfinding, movement, eating, pregnancy, death
- `src/systems/setup.rs` — world gen (Perlin), initial spawns, UI
- `src/systems/graphics.rs` — visuals and UI updates
- `src/systems/input.rs` — camera controls, click interactions


