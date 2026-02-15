[![Build and Test](https://github.com/twitchax/relativity/actions/workflows/build.yml/badge.svg)](https://github.com/twitchax/relativity/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/twitchax/relativity/branch/main/graph/badge.svg?token=35MZN0YFZF)](https://codecov.io/gh/twitchax/relativity)
[![GitHub all releases](https://img.shields.io/github/downloads/twitchax/relativity/total?label=binary)](https://github.com/twitchax/relativity/releases)
[![Rust](https://img.shields.io/badge/rust-nightly-blue.svg)](https://github.com/twitchax/relativity)
[![License:MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# relativity

A [bevy](https://bevyengine.org/) game designed to explore the effects of relativity on the player, oftentimes with the requirement that relativity must be used to solve the puzzles.

## Install

**Note:** You also need to download the assets folder alongside the binary for the game to run properly.

### Windows

```powershell
$ iwr https://github.com/twitchax/relativity/releases/latest/download/relativity_x86_64-pc-windows-gnu.zip
$ Expand-Archive relativity_x86_64-pc-windows-gnu.zip -DestinationPath C:\Users\%USERNAME%\AppData\Local\Programs\relativity
```

### Mac OS (Apple Silicon)

```bash
$ curl -LO https://github.com/twitchax/relativity/releases/latest/download/relativity_aarch64-apple-darwin.zip
$ unzip relativity_aarch64-apple-darwin.zip -d /usr/local/bin
$ chmod a+x /usr/local/bin/relativity
```

### Linux

```bash
$ curl -LO https://github.com/twitchax/relativity/releases/latest/download/relativity_x86_64-unknown-linux-gnu.zip
$ unzip relativity_x86_64-unknown-linux-gnu.zip -d /usr/local/bin
$ chmod a+x /usr/local/bin/relativity
```

## Usage

### Play in Browser

You can play the game directly in your browser at:

**https://relativity.gh.twitchax.com/**

The web version is automatically built and deployed from the main branch.

### Run the Game

After installing the binary and assets, run:

```bash
$ relativity
```

Or, if building from source:

```bash
$ cargo make build && cargo run
```

### Controls

- **Hover** to preview your aim direction (dotted line from the player toward the cursor).
- **Click** to lock your launch angle.
- **Drag** to set launch power — a radial arc around the player fills from cyan (0.1 c) to red (0.99 c) with tick marks at 0.25 c, 0.5 c, 0.75 c, and 0.9 c.
- **Release** to fire.
- **Right-click** or **Escape** to cancel the launch from any phase.
- **Space** to pause/resume the simulation.
- **+/−** to adjust simulation speed (0.25×–2.00×).
- **G** to toggle the gravity grid display.
- **Escape** to return to the menu from any game state.

## Development

For detailed development instructions, including how to set up your environment, build the project, add new levels, and contribute, see [DEVELOPMENT.md](DEVELOPMENT.md).

Quick start:

```bash
# Clone the repository
$ git clone https://github.com/twitchax/relativity.git
$ cd relativity

# Run in debug mode
$ cargo run

# Run tests
$ cargo make test

# Format code
$ cargo make fmt
```

### Building for Web (WASM)

To build and test the web version locally:

```bash
# Add the WASM target
$ rustup target add wasm32-unknown-unknown

# Build for deployment
$ cargo make build-web
# Output will be in the dist/ directory
```

The web version is automatically deployed to GitHub Pages when changes are pushed to the main branch.

## Test

```bash
$ cargo make test
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT