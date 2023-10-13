[![Build and Test](https://github.com/twitchax/relativity/actions/workflows/build.yml/badge.svg)](https://github.com/twitchax/relativity/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/twitchax/relativity/branch/main/graph/badge.svg?token=35MZN0YFZF)](https://codecov.io/gh/twitchax/relativity)
[![Version](https://img.shields.io/crates/v/relativity.svg)](https://crates.io/crates/relativity)
[![GitHub all releases](https://img.shields.io/github/downloads/twitchax/relativity/total?label=binary)](https://github.com/twitchax/relativity/releases)
[![Rust](https://img.shields.io/badge/rust-nightly-blue.svg?maxAge=3600)](https://github.com/twitchax/relativity)
[![License:MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# relativity

A [bevy](https://bevyengine.org/) game designed to explore the effects of relativity on the player, oftentimes with the requirement that relativity must be used to solve the puzzles.

## Binary Usage

### Install

At present, you also need to install the assets by downloading the folder next to the binary.

Windows:

```powershell
$ iwr https://github.com/twitchax/relativity/releases/latest/download/relativity_x86_64-pc-windows-gnu.zip
$ Expand-Archive relativity_x86_64-pc-windows-gnu.zip -DestinationPath C:\Users\%USERNAME%\AppData\Local\Programs\relativity
```

Mac OS (Apple Silicon):

```bash
$ curl -LO https://github.com/twitchax/relativity/releases/latest/download/relativity_aarch64-apple-darwin.zip
$ unzip relativity_aarch64-apple-darwin.zip -d /usr/local/bin
$ chmod a+x /usr/local/bin/relativity
```

Linux:

```bash
$ curl -LO https://github.com/twitchax/relativity/releases/latest/download/relativity_x86_64-unknown-linux-gnu.zip
$ unzip relativity_x86_64-unknown-linux-gnu.zip -d /usr/local/bin
$ chmod a+x /usr/local/bin/relativity
```

## Development

### Run in Debug Mode

```bash
$ cargo run
```

### Add a Level

The levels are defined in `src/game/levels/mod.rs`.

First, you need to add a new enum value.

```rust
#[derive(Resource, Default)]
pub enum CurrentLevel {
    #[default]
    One,
    // New one.
    Two,
}
```

If you're just testing it out, make it the default.

```rust
#[derive(Resource, Default)]
pub enum CurrentLevel {
    #[default]
    One,
    Two,
    // New one.
    #[default]
    Three,
}
```

Then, add the mapping to the `spawn_level` function.

```rust
pub fn spawn_level(commands: Commands, asset_server: Res<AssetServer>, current_level: Res<CurrentLevel>) {
    match current_level.into_inner() {
        CurrentLevel::One => level1(commands, asset_server),
        // New one.
        CurrentLevel::Two => level2(commands, asset_server),
    }
}
```

Finally, add a new level function with the proper signature.

```rust
fn level2(commands: Commands, asset_server: Res<AssetServer>) {
    // ...
}
```

You can take a look at `level1` as an example.

## Test

Not yet.

## Bench

Not yet.

## License

MIT