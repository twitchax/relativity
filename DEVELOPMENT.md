# relativity development

## Local Setup

### Prerequisites

Make sure you have Rust (nightly) installed via [rustup](https://rustup.rs/).

```bash
$ rustup install nightly
```

### System Dependencies

For Linux development, you'll need several system libraries for Bevy's rendering and audio support:

```bash
$ sudo apt-get update && sudo apt-get install -y \
    libasound2-dev \
    portaudio19-dev \
    build-essential \
    libpulse-dev \
    libdbus-1-dev \
    libudev-dev \
    libwayland-dev \
    libxkbcommon-dev
```

For other platforms, refer to the [Bevy setup guide](https://bevyengine.org/learn/quick-start/getting-started/setup/).

## Build and Test

### Build

```bash
$ cargo build
```

### Run in Debug Mode

```bash
$ cargo run
```

### Run in Release Mode

```bash
$ cargo run --release
```

### Format Code

```bash
$ cargo fmt
```

### Lint with Clippy

```bash
$ cargo clippy --all-targets --all-features
```

### Test

```bash
$ cargo test
```

## Development Workflow

### Add a Level

The levels are defined in `src/game/levels/mod.rs`.

First, add a new enum value to `CurrentLevel`:

```rust
#[derive(Resource, Default)]
pub enum CurrentLevel {
    #[default]
    One,
    Two,
    // New level
    Three,
}
```

If you're testing it out, make it the default:

```rust
#[derive(Resource, Default)]
pub enum CurrentLevel {
    One,
    Two,
    // New level (set as default for testing)
    #[default]
    Three,
}
```

Then, add the mapping in the `spawn_level` function:

```rust
pub fn spawn_level(commands: Commands, asset_server: Res<AssetServer>, current_level: Res<CurrentLevel>) {
    match current_level.into_inner() {
        CurrentLevel::One => level1(commands, asset_server),
        CurrentLevel::Two => level2(commands, asset_server),
        // New level
        CurrentLevel::Three => level3(commands, asset_server),
    }
}
```

Finally, add a new level function with the proper signature:

```rust
fn level3(commands: Commands, asset_server: Res<AssetServer>) {
    // Level implementation
}
```

You can take a look at `level1` as an example.

## Release and Publish

### Building Release Binaries

The GitHub Actions workflow automatically builds release binaries for Windows, macOS, and Linux when code is pushed to the `main` branch.

Binaries are uploaded as artifacts and can be downloaded from the Actions tab.

### Manual Release Build

```bash
# Linux
$ cargo build --release --target x86_64-unknown-linux-gnu

# Windows (cross-compile from Linux)
$ cargo install cross
$ cross build --release --target x86_64-pc-windows-gnu

# macOS (requires macOS host for aarch64-apple-darwin)
$ cargo build --release --target aarch64-apple-darwin
```

### Publishing to crates.io

Currently, this is a binary game project and is not published to crates.io. If you want to publish it in the future:

```bash
$ cargo publish
```

## Assets

The game assets are stored in the `assets/` directory and must be present next to the binary when distributing the game.
