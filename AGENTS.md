# Agents Guide

This document provides guidance for AI coding agents working in this repository.

## Workspace Overview

- `src/`: Main source code
- `.mr/`: microralph state directory
  - `prds/`: PRD files
  - `templates/`: PRD templates
  - `prompts/`: Static prompt files for each stage
  - `PRDS.md`: Auto-generated PRD index

## Quick Start

```bash
# Build
cargo make build

# Test (uses nextest)
cargo make test

# Full CI (fmt-check, clippy, nextest)
cargo make ci

# UAT (the one true gate)
cargo make uat
```

## Available `cargo make` Tasks

All dev/CI workflows route through `cargo make`. Key tasks:

| Task | Description |
|------|-------------|
| `fmt` | Format code with rustfmt |
| `fmt-check` | Check formatting without modifying |
| `clippy` | Run clippy with `-D warnings` |
| `build` | Debug build |
| `build-release` | Release build |
| `test` | Run tests with nextest |
| `test-cargo` | Fallback: run tests with `cargo test` |
| `ci` | Full CI pipeline (fmt-check + clippy + test) |
| `uat` | The one true gate (runs `ci`) |
| `codecov` | Generate LCOV coverage report |
| `codecov-html` | Generate HTML coverage report |
| `build-linux` | Cross-compile for `x86_64-unknown-linux-gnu` |
| `build-windows` | Cross-compile for `x86_64-pc-windows-gnu` |
| `build-macos` | Build for `aarch64-apple-darwin` |
| `build-web` | Build WASM via Trunk |
| `changelog` | Generate CHANGELOG.md via git-cliff |
| `release` | Full release pipeline (CI → changelog → bump → push) |
| `github-release` | Create GitHub release with artifacts |
| `publish-all` | Full publish pipeline (GitHub release) |
| `clean` | Run `cargo clean` |

## Conventions for Agents

- Keep changes minimal and focused; avoid unrelated refactors.
- Follow existing style; don't add license headers.
- Use `anyhow::Result` for fallible functions.
- Prefer `tracing` over `println!` for diagnostics.
- All dev commands route through `cargo make` (never raw `cargo test`, `cargo clippy`, etc.).
- Tests run via [nextest](https://nexte.st/); configuration is in `.config/nextest.toml`.

### Code Style

- Use vertical whitespace generously to separate logical sections.
- Prefer explicitness over implicitness.
- Reduce nesting by using guard clauses and early returns.
- Prefer functional programming techniques where appropriate.

### Clippy Denies

Strict clippy denies are enforced in `src/main.rs`:
- `#![deny(unused)]`, `#![deny(clippy::unwrap_used)]`, `#![deny(clippy::correctness)]`, `#![deny(clippy::complexity)]`, `#![deny(clippy::pedantic)]`
- `clippy::needless_pass_by_value` is globally allowed (`#![allow]` in `lib.rs`) since Bevy's ECS requires pass-by-value for system parameters.
- Use `#[allow(clippy::cast_possible_truncation)]` on intentional `f64 as f32` conversions (e.g., graphics/pixel coordinates).
- Use `f64::from(f32_value)` instead of `f32_value as f64` for lossless casts.
- Use `std::sync::LazyLock` instead of `once_cell::sync::Lazy` for lazy statics.

### UI Architecture

- In-game HUD uses **bevy_lunex** (v0.6) for layout and rendering (`src/game/hud/mod.rs`). Menu and outcome screens use native `bevy_ui`.
- `UiLunexPlugins` is registered in `src/main.rs` (not inside `GamePlugin`) because it requires `DefaultPlugins` resources. This avoids panics in headless tests that use `MinimalPlugins`.
- HUD text labels use individual marker components (`HudPlayerTime`, `HudVelocityGamma`, etc.) for targeted queries rather than a single concatenated text entity.

### CI / Workflow Patterns

- CI workflow (`.github/workflows/build.yml`) uses `cargo-make` tasks, not raw cargo commands. All jobs (test, codecov, platform builds, web deploy) are in this single file.
- All Linux CI jobs install `clang` and `mold` and set `RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=mold"` to avoid linker OOM when building 45+ test binaries with Bevy.
- Rust toolchain is pinned in `rust-toolchain.toml` (single source of truth). Workflow files read the channel dynamically via a "Read toolchain" step — no hardcoded toolchain dates in workflow YAML.
- Tools are installed via `cargo-binstall` (fast binary installs), not `cargo install`.
- GitHub Pages deployment uses `peaceiris/actions-gh-pages@v4` in the `build_web` job (requires `permissions: contents: write`).
- `.github/workflows/copilot-setup-steps.yml` sets up the environment for GitHub Copilot coding agents.
- Changelog is generated with [git-cliff](https://git-cliff.org/) using `cliff.toml` config.

## PRD Format

PRDs are Markdown files with YAML frontmatter containing:

- `id`: Unique identifier (e.g., PRD-0001)
- `title`: Human-readable title
- `status`: draft | active | done | parked
- `tasks`: List of tasks with id, title, priority, status

History entries are appended by `mr run` at the bottom of the PRD.

---

## Manual Updates by Agents

Automatic AGENTS.md updates have been removed to give agents more flexibility. Agents should update AGENTS.md manually when:

- Discovering new build/test commands or troubleshooting steps
- Identifying code patterns or conventions not already documented
- Adding new tools or dependencies that affect the workflow
- Finding solutions to common issues during implementation

Update any relevant section, not just this one. Keep additions concise and actionable.
