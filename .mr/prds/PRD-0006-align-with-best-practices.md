---
id: PRD-0006
title: "Align with Best Practices: Makefile, CI, Denies, and Docs"
status: active
owner: twitchax
created: 2026-02-07
updated: 2026-02-07
principles:
  - Align CI, tooling, and lint configuration with twitchax/kord and twitchax/microralph patterns
  - Use cargo-make as the single entry point for all dev/CI workflows
  - Enforce strict clippy denies to maintain code quality
  - Keep changes mechanical where possible to reduce risk
references:
  - name: "microralph Makefile.toml"
    url: "https://github.com/twitchax/microralph/blob/main/Makefile.toml"
  - name: "microralph build.yml"
    url: "https://github.com/twitchax/microralph/blob/main/.github/workflows/build.yml"
  - name: "kord Makefile.toml"
    url: "https://github.com/twitchax/kord/blob/main/Makefile.toml"
  - name: "kord build.yml"
    url: "https://github.com/twitchax/kord/blob/main/.github/workflows/build.yml"
  - name: "microralph main.rs denies"
    url: "https://github.com/twitchax/microralph/blob/main/src/main.rs"
acceptance_tests:
  - id: uat-001
    name: "cargo make ci passes cleanly (fmt-check, clippy with denies, nextest)"
    command: cargo make ci
    uat_status: unverified
  - id: uat-002
    name: "cargo make uat passes cleanly"
    command: cargo make uat
    uat_status: unverified
  - id: uat-003
    name: "All clippy denies are satisfied with zero warnings"
    command: cargo clippy --all-targets --all-features -- -D warnings
    uat_status: unverified
tasks:
  - id: T-001
    title: "Create Makefile.toml with full task suite"
    priority: 1
    status: done
    notes: "Model after microralph: tool installs (nextest, llvm-cov, git-cliff), fmt, fmt-check, clippy, build, build-release, test (nextest), ci, uat, codecov, platform builds, changelog, release, github-release, clean"
  - id: T-002
    title: "Create .config/nextest.toml"
    priority: 1
    status: todo
    notes: "Copy microralph pattern: slow-timeout with period 5s, terminate-after 6"
  - id: T-003
    title: "Add clippy denies to src/main.rs"
    priority: 2
    status: todo
    notes: "Add deny(unused), deny(clippy::unwrap_used), deny(clippy::correctness), deny(clippy::complexity), deny(clippy::pedantic) matching microralph"
  - id: T-004
    title: "Fix all clippy deny violations across codebase"
    priority: 2
    status: todo
    notes: "Fix unwrap_used (use expect or proper error handling), pedantic lints, complexity warnings, and unused items. May require allow attributes on specific items where pedantic is too strict (e.g., Bevy system signatures)."
  - id: T-005
    title: "Rewrite build.yml to use cargo-make and match microralph pattern"
    priority: 3
    status: todo
    notes: "Pin RUST_TOOLCHAIN, use cargo-binstall for tool installs, single ci task, cache-bin false, add rustfmt+clippy components, use cargo make tasks for platform builds"
  - id: T-006
    title: "Update web.yml to use cargo-make and cargo-binstall for Trunk"
    priority: 3
    status: todo
    notes: "Use cargo-binstall for trunk install, add a build-web task to Makefile.toml"
  - id: T-007
    title: "Add copilot-setup-steps.yml workflow"
    priority: 3
    status: todo
    notes: "Model after kord: checkout, install system deps (Bevy audio/graphics libs), rust toolchain, cargo-binstall, rust-cache, install cargo tools, cargo fetch"
  - id: T-008
    title: "Add codecov job to build.yml"
    priority: 4
    status: todo
    notes: "Add codecov job using cargo make codecov, upload with codecov/codecov-action"
  - id: T-009
    title: "Add release and changelog tasks to Makefile.toml"
    priority: 4
    status: todo
    notes: "git-cliff for changelog generation, github-release task for creating GitHub releases with binary artifacts from CI. No crates.io publishing needed."
  - id: T-010
    title: "Update DEVELOPMENT.md to reference cargo make commands"
    priority: 5
    status: todo
    notes: "Replace raw cargo commands with cargo make equivalents throughout"
  - id: T-011
    title: "Update CONTRIBUTING.md to reference cargo make commands"
    priority: 5
    status: todo
    notes: "Replace cargo test / cargo clippy / cargo fmt with cargo make ci"
  - id: T-012
    title: "Update AGENTS.md if new patterns or workflows are introduced"
    priority: 5
    status: todo
    notes: "Document cargo make tasks, deny patterns, and any new conventions"
---

# Summary

Align the relativity repository's developer tooling, CI pipeline, lint configuration, and documentation with the patterns established in twitchax/kord and twitchax/microralph. This brings consistency across projects and enforces stricter code quality standards.

# Problem

The relativity repo currently uses raw `cargo` commands in CI and documentation, has no `Makefile.toml`, no clippy deny directives, and no standardized release/changelog workflow. This diverges from the patterns in kord and microralph, making it harder to maintain consistency across projects and allowing code quality issues to slip through.

Key gaps:
- No `Makefile.toml` — all workflows use raw cargo commands
- No clippy denies — `unwrap()`, pedantic issues, and unused code go unchecked
- CI uses `cargo install` instead of `cargo-binstall`, no pinned toolchain, separate clippy job instead of unified `ci` task
- No nextest, no changelog generation, no release automation
- No `copilot-setup-steps.yml` for Copilot coding agent
- Documentation references raw `cargo` commands instead of `cargo make`

# Goals

1. Create a `Makefile.toml` that mirrors the microralph structure with all standard tasks (fmt, clippy, test, ci, uat, codecov, platform builds, release, changelog)
2. Add strict clippy denies (`unused`, `unwrap_used`, `correctness`, `complexity`, `pedantic`) and fix all violations
3. Rewrite CI workflows to use `cargo-make`, `cargo-binstall`, pinned toolchain, and proper caching
4. Add codecov integration, changelog generation (git-cliff), and release automation (GitHub releases with binary artifacts)
5. Add `copilot-setup-steps.yml` for Copilot coding agent support
6. Update all documentation to reference `cargo make` commands

# Technical Approach

## Makefile.toml

Create `Makefile.toml` following the microralph structure with sections:
- **Tool Installation**: nextest, llvm-cov, git-cliff via `cargo-binstall`
- **Formatting**: `fmt` and `fmt-check` tasks
- **Linting**: `clippy` with `-D warnings`
- **Build**: `build`, `build-release`, platform-specific builds (linux, windows, macos)
- **Testing**: `test` (nextest), `test-cargo` (fallback)
- **CI Pipeline**: `ci` = fmt-check + clippy + test
- **UAT**: `uat` = ci pipeline (the one true gate)
- **Coverage**: `codecov` and `codecov-html` via cargo-llvm-cov
- **Release**: `changelog` (git-cliff), `release`, `github-release`
- **Web**: `build-web` (trunk build for WASM/GitHub Pages)
- **Clean**: `clean`

## Clippy Denies

Add to `src/main.rs`:
```rust
#![deny(unused)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::correctness)]
#![deny(clippy::complexity)]
#![deny(clippy::pedantic)]
```

Then fix all violations. Expected categories:
- `unwrap_used`: Replace `unwrap()` calls with `expect()` or proper error handling
- `pedantic`: Missing docs on public items, needless pass-by-value, type complexity, etc.
- Module-level `#[allow(...)]` where pedantic is genuinely too strict (e.g., Bevy system parameter signatures)

## CI Workflow Rewrite

```
build.yml
├── test (cargo make ci)
├── codecov (cargo make codecov → codecov-action)
├── build_linux (cargo make build-linux → upload-artifact)
├── build_windows (cargo make build-windows → upload-artifact)
└── build_macos (cargo make build-macos → upload-artifact)

web.yml (unchanged structure, but use cargo-binstall for trunk)

copilot-setup-steps.yml (new)
├── checkout
├── install system deps (Bevy audio/graphics libs)
├── rust toolchain (pinned nightly)
├── cargo-binstall
├── rust-cache
├── install cargo tools
└── cargo fetch
```

# Assumptions

- The nightly Rust toolchain will remain the project's default
- Bevy system dependencies on Linux remain stable across CI runs
- `cargo-binstall` availability in CI (via cargo-bins/cargo-binstall@main action)
- Some pedantic clippy lints may need `#[allow(...)]` on specific Bevy-idiomatic patterns

# Constraints

- This is a binary game project — no crates.io publishing
- WASM builds use Trunk (not wasm-pack), so the web workflow pattern differs from kord/microralph
- Bevy's ECS system signatures may trigger pedantic lints that need targeted allows
- Existing `#[allow(clippy::type_complexity)]` on the gravitational constant may need to remain

# References to Code

- `src/main.rs` — Entry point where denies will be added
- `src/game/shared/constants.rs` — Has `#[allow(clippy::type_complexity)]` that may need adjustment
- `.github/workflows/build.yml` — Current CI workflow to be rewritten
- `.github/workflows/web.yml` — WASM/GitHub Pages workflow to be updated
- `DEVELOPMENT.md` — Developer docs to update
- `CONTRIBUTING.md` — Contributor docs to update
- `AGENTS.md` — Agent docs to update
- `Cargo.toml` — May need metadata additions
- `rustfmt.toml` — Existing formatting config (no changes expected)

# Non-Goals (MVP)

- Adding unit tests for physics/game logic (separate effort)
- Refactoring code architecture or game systems
- Adding `tracing` / structured logging
- Fixing game bugs or improving gameplay
- Publishing to crates.io
- Docker/container builds
- Devcontainer configuration

# History

## 2026-02-07 — T-001 Completed
- **Task**: Create Makefile.toml with full task suite
- **Status**: ✅ Done
- **Changes**:
  - Created `Makefile.toml` with full task suite modeled after microralph: tool installs (nextest, llvm-cov, git-cliff, cargo-release, trunk), fmt, fmt-check, clippy, build, build-release, test (nextest), test-cargo (fallback), ci, uat, codecov, codecov-html, platform builds (linux, windows, macos), build-web (trunk), changelog, release, github-release, clean
  - Created `.config/nextest.toml` with slow-timeout configuration (period 5s, terminate-after 6)
  - `cargo make uat` passes: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations.

