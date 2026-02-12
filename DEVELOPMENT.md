# relativity development

## Local Setup

### Prerequisites

Make sure you have Rust (nightly) installed via [rustup](https://rustup.rs/).

```bash
$ rustup install nightly
```

You will also need [cargo-make](https://github.com/sagiegurari/cargo-make) and [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) installed:

```bash
$ cargo install cargo-make
$ cargo install cargo-binstall
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
$ cargo make build
```

### Run in Debug Mode

```bash
$ cargo run
```

### Run in Release Mode

```bash
$ cargo make build-release
```

### Format Code

```bash
$ cargo make fmt
```

### Lint with Clippy

```bash
$ cargo make clippy
```

### Test

```bash
$ cargo make test
```

### Full CI Pipeline (Format Check + Clippy + Test)

```bash
$ cargo make ci
```

### UAT (The One True Gate)

```bash
$ cargo make uat
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
$ cargo make build-linux

# Windows (cross-compile from Linux)
$ cargo make build-windows

# macOS (requires macOS host for aarch64-apple-darwin)
$ cargo make build-macos
```

### Changelog

Generate a changelog using [git-cliff](https://git-cliff.org/):

```bash
$ cargo make changelog
```

### Release

Run the full automated release pipeline (CI → changelog → version bump → push → wait for CI artifacts):

```bash
$ cargo make release
```

### GitHub Release

Create a GitHub release with binary artifacts:

```bash
$ cargo make github-release <tag>
```

### Code Coverage

Generate code coverage reports:

```bash
# LCOV format (for CI / codecov upload)
$ cargo make codecov

# HTML format (for local viewing)
$ cargo make codecov-html
```

## Building for Web (WASM)

To build the web version:

```bash
# Build for deployment (installs trunk automatically via cargo-binstall)
$ cargo make build-web
# Output will be in the dist/ directory
```

The web version is automatically deployed to GitHub Pages when changes are pushed to the main branch.

## Assets

The game assets are stored in the `assets/` directory and must be present next to the binary when distributing the game.

## TODO

- Even better chrome a the bottom.
- Better launch controls.
- Add more levels.