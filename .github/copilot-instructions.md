# Project Overview

This repository contains **relativity**, a game built with the [Bevy](https://bevyengine.org/) game engine that explores the effects of relativity on the player. The game features puzzles that require understanding and using relativistic effects to solve. Players navigate through levels where time dilation and other relativistic phenomena affect gameplay mechanics.

## Tech Stack

- **Language**: Rust (nightly toolchain)
- **Game Engine**: Bevy 0.17
- **Physics/Math Libraries**:
  - `uom` (0.37) - Units of measurement
  - `nalgebra-spacetime` (0.5) - Spacetime calculations
  - `glam` (0.30) - Graphics linear algebra math
  - `once_cell` (1) - Lazy static initialization

## Build and Run

### Prerequisites
- Rust nightly toolchain
- Linux: System dependencies for audio and graphics
  ```bash
  sudo apt-get update && sudo apt-get install -y libasound2-dev portaudio19-dev build-essential libpulse-dev libdbus-1-dev libudev-dev libwayland-dev libxkbcommon-dev
  ```

### Commands
- **Install dependencies**: Dependencies are managed by Cargo automatically
- **Run in debug mode**: `cargo run`
- **Build for release**: `cargo build --release`
- **Run tests**: `cargo test`
- **Format code**: `cargo fmt`
- **Check code**: `cargo check`

### Supported Platforms
- Linux (x86_64-unknown-linux-gnu)
- Windows (x86_64-pc-windows-gnu)
- macOS (aarch64-apple-darwin)

## Test Instructions

The project uses Rust's built-in test framework. Run tests with:
```bash
cargo test
```

Note: As mentioned in the README, comprehensive test coverage is still being developed.

## Coding Guidelines

### Rust Formatting (rustfmt.toml)
- Maximum line width: 140 characters
- Struct literal width: 40 characters
- Reorder impl items: enabled
- Format macro bodies: disabled
- Format code in doc comments: enabled

### Code Conventions
- Use Bevy's ECS (Entity Component System) patterns
- Follow Rust naming conventions:
  - `snake_case` for functions and variables
  - `PascalCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants
- Prefer explicit type annotations for clarity when working with complex game logic
- Use Bevy's system parameters (Commands, Query, Res, etc.) appropriately
- Game entities should be marked with the `GameItem` component for proper lifecycle management

### Architecture Patterns
- Use Bevy plugins to organize features (e.g., `MenuPlugin`, `GamePlugin`)
- State management through Bevy's `State` system (see `AppState`)
- Resource-based configuration (e.g., `CurrentLevel`)
- Systems should be pure functions that operate on components through queries

## Directory Structure

- `/src/main.rs` - Application entry point, configures Bevy app with plugins
- `/src/game/` - Core game logic
  - `/src/game/levels/` - Level definitions and spawn functions
  - `/src/game/player/` - Player entity, sprite, and clock systems
  - `/src/game/destination/` - Level completion/destination logic
  - `/src/game/object/` - Static game objects (planets, etc.)
  - `/src/game/observer/` - Observer clock for relativity comparisons
  - `/src/game/shared/` - Shared game utilities, constants, types, and helper functions
- `/src/menu/` - Menu system and UI
- `/src/shared/` - Application-wide shared code (state management, types)
- `/assets/` - Game assets (sprites, fonts, etc.)
- `.github/workflows/` - CI/CD configuration for multi-platform builds

## Adding New Levels

When adding a new level:
1. Add a new enum variant to `CurrentLevel` in `src/game/levels/mod.rs`
2. Update the `spawn_level` match expression to handle the new variant
3. Create a new level function with signature: `fn level_n(mut commands: Commands, asset_server: Res<AssetServer>)`
4. Use existing level functions as templates
5. Make the new level the default temporarily for testing by adding `#[default]` attribute

## Resources

- [Bevy Engine Documentation](https://bevyengine.org/learn/)
- [Bevy API Docs](https://docs.rs/bevy/latest/bevy/)
- [README.md](../README.md) - User-facing documentation and installation instructions
- [Cargo.toml](../Cargo.toml) - Project dependencies and metadata
