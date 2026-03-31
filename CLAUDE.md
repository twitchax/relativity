# Relativity

A Bevy game exploring relativistic effects through puzzles. Players navigate levels where time dilation and other relativistic phenomena affect gameplay mechanics.

## Tech Stack

- **Language**: Rust (nightly, pinned in `rust-toolchain.toml`)
- **Game Engine**: Bevy 0.18 (minimal feature set, not `default_plugins`)
- **UI**: `bevy_lunex` 0.6 (advanced UI rendering for HUD)
- **Camera Effects**: `bevy_trauma_shake` 0.7
- **Physics/Math**: `uom` 0.37 (SI units), `nalgebra-spacetime` 0.5, `glam` 0.31
- **Task Runner**: `cargo-make` (see `Makefile.toml`)

## Build and Run

### Prerequisites
- Rust nightly toolchain (version pinned in `rust-toolchain.toml`)
- Linux system deps: `sudo apt-get install -y libasound2-dev portaudio19-dev build-essential libpulse-dev libdbus-1-dev libudev-dev libwayland-dev libxkbcommon-dev`

### Commands
- **Run**: `cargo run`
- **Build release**: `cargo build --release`
- **Test**: `cargo make test` (nextest)
- **Lint**: `cargo make clippy`
- **Format check**: `cargo make fmt-check`
- **Format fix**: `cargo make fmt`
- **Full CI pipeline**: `cargo make ci` (fmt-check + clippy + test)
- **UAT gate**: `cargo make uat` (alias for `ci`)
- **Coverage**: `cargo make codecov` (lcov) or `cargo make codecov-html`
- **Web build**: `cargo make build-web` (WASM via Trunk)
- **Web dev server**: `cargo make serve-web-dev`
- **Web release server**: `cargo make serve-web` (wasm-release profile)

### Supported Platforms
- Linux (x86_64-unknown-linux-gnu)
- Windows (x86_64-pc-windows-gnu, cross-compiled)
- macOS (aarch64-apple-darwin)
- Web (wasm32-unknown-unknown via Trunk, deployed to GitHub Pages)

## Code Conventions

### Formatting (rustfmt.toml)
- Max line width: **200** characters
- Struct literal width: 40 characters
- Reorder impl items: enabled
- Format macro bodies: disabled

### Bevy ECS Patterns
- Use Bevy plugins to organize features (e.g., `MenuPlugin`, `GamePlugin`, `HudPlugin`)
- Two-level state: `AppState` (Menu / InGame) and `GameState` (Paused / Running / SimPaused / Finished / Failed)
- Resource-based configuration: `CurrentLevel`, `SimRate`, `GridVisible`, `LaunchState`, `FadeState`
- Systems are gated by state combinations via `.run_if(in_state(...))`
- All game entities must carry the `GameItem` marker component for lifecycle cleanup on level transitions
- System ordering is explicit where needed (e.g., `position_update.after(velocity_update)`)

### Physics
- All physical quantities use `uom` SI types (`UomLength`, `UomMass`, `UomVelocity`, etc.)
- Gravity uses Plummer softening to prevent singularities
- Velocity clamped to 0.9999c
- Time dilation: `dt_player = dt / (γ_v · γ_g)` where γ_v is velocity Lorentz factor and γ_g is gravitational gamma (product over all masses)

### Testing
- 51 E2E tests in `/tests/`, common helpers in `tests/common/`
- Tests use headless Bevy (MinimalPlugins) — no window required
- Visual regression baselines in `tests/baselines/`
- Dev-dependencies: `approx`, `image` (png), `proptest`

## Architecture

```
src/
├── main.rs              # App entry: DefaultPlugins + MenuPlugin + GamePlugin
├── lib.rs               # Module exports
├── shared/
│   ├── state.rs         # AppState, GameState enums
│   └── types.rs         # Camera spawning
├── menu/
│   └── mod.rs           # Menu UI, level buttons, auto-advance (PendingNextLevel)
└── game/
    ├── mod.rs           # GamePlugin: all system scheduling
    ├── shared/
    │   ├── types.rs     # Core components: Position, Velocity, Radius, Mass, Clock,
    │   │                #   VelocityGamma, GravitationalGamma, LaunchState, SimRate, etc.
    │   ├── constants.rs # Physical constants (UNIT_RADIUS, MASS_OF_SUN, SPEED_OF_LIGHT, etc.)
    │   ├── helpers.rs   # Collision detection, coordinate transforms, scale math
    │   └── systems.rs   # Physics: gravity, velocity/position integration, collision checks,
    │                    #   input handlers (escape, pause, grid toggle, sim rate)
    ├── levels/
    │   └── mod.rs       # CurrentLevel enum, spawn_level, level definitions, reset logic
    ├── player/
    │   ├── player_clock.rs  # Lorentz gamma calculations, player time dilation
    │   ├── player_sprite.rs # Launch state machine (Idle → AimLocked → Launching),
    │   │                    #   preview lines, arc ticks, readout
    │   └── shared.rs        # Player marker component
    ├── observer/
    │   └── mod.rs       # Observer clock (coordinate time, no dilation)
    ├── object/
    │   └── mod.rs       # StaticPlanetBundle, DynamicPlanetBundle
    ├── destination/
    │   └── mod.rs       # DestinationBundle
    ├── hud/
    │   └── mod.rs       # HudPlugin: cockpit-style panels via bevy_lunex,
    │                    #   flash-on-change, gamma-based coloring, glow pulse
    ├── trail/
    │   └── mod.rs       # Ring buffer trail (max 2000 pts), gamma→color gradient
    ├── gravity_grid/
    │   └── mod.rs       # 40×24 warped grid, log-scaled displacement, gizmo rendering
    ├── outcome/
    │   └── mod.rs       # Success/failure overlays, auto-reset timer, camera shake
    └── fade/
        └── mod.rs       # Full-screen fade transitions between states
```

## PRDs

PRDs are managed by [microralph](https://github.com/twitchax/microralph) in `.mr/prds/`. All 14 PRDs (PRD-0001 through PRD-0014) are status `done`. Config is in `.mr/config.toml`.

## Adding New Levels

1. Add a new enum variant to `CurrentLevel` in `src/game/levels/mod.rs`
2. Update `CurrentLevel::all()` and `CurrentLevel::next()`
3. Add a `Display` impl arm
4. Update the `spawn_level` match expression
5. Create a new level function: `fn level_n(mut commands: Commands, asset_server: Res<AssetServer>)`
6. Use existing level functions as templates (percentage-based coordinates)
7. Add E2E tests for the new level in `/tests/`
