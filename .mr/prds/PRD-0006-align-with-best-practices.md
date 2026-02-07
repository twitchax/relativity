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
    uat_status: verified
  - id: uat-002
    name: "cargo make uat passes cleanly"
    command: cargo make uat
    uat_status: verified
  - id: uat-003
    name: "All clippy denies are satisfied with zero warnings"
    command: cargo clippy --all-targets --all-features -- -D warnings
    uat_status: verified
tasks:
  - id: T-001
    title: "Create Makefile.toml with full task suite"
    priority: 1
    status: done
    notes: "Model after microralph: tool installs (nextest, llvm-cov, git-cliff), fmt, fmt-check, clippy, build, build-release, test (nextest), ci, uat, codecov, platform builds, changelog, release, github-release, clean"
  - id: T-002
    title: "Create .config/nextest.toml"
    priority: 1
    status: done
    notes: "Copy microralph pattern: slow-timeout with period 5s, terminate-after 6"
  - id: T-003
    title: "Add clippy denies to src/main.rs"
    priority: 2
    status: done
    notes: "Add deny(unused), deny(clippy::unwrap_used), deny(clippy::correctness), deny(clippy::complexity), deny(clippy::pedantic) matching microralph"
  - id: T-004
    title: "Fix all clippy deny violations across codebase"
    priority: 2
    status: done
    notes: "Fix unwrap_used (use expect or proper error handling), pedantic lints, complexity warnings, and unused items. May require allow attributes on specific items where pedantic is too strict (e.g., Bevy system signatures)."
  - id: T-005
    title: "Rewrite build.yml to use cargo-make and match microralph pattern"
    priority: 3
    status: done
    notes: "Pin RUST_TOOLCHAIN, use cargo-binstall for tool installs, single ci task, cache-bin false, add rustfmt+clippy components, use cargo make tasks for platform builds"
  - id: T-006
    title: "Update web.yml to use cargo-make and cargo-binstall for Trunk"
    priority: 3
    status: done
    notes: "Use cargo-binstall for trunk install, add a build-web task to Makefile.toml"
  - id: T-007
    title: "Add copilot-setup-steps.yml workflow"
    priority: 3
    status: done
    notes: "Model after kord: checkout, install system deps (Bevy audio/graphics libs), rust toolchain, cargo-binstall, rust-cache, install cargo tools, cargo fetch"
  - id: T-008
    title: "Add codecov job to build.yml"
    priority: 4
    status: done
    notes: "Add codecov job using cargo make codecov, upload with codecov/codecov-action"
  - id: T-009
    title: "Add release and changelog tasks to Makefile.toml"
    priority: 4
    status: done
    notes: "git-cliff for changelog generation, github-release task for creating GitHub releases with binary artifacts from CI. No crates.io publishing needed."
  - id: T-010
    title: "Update DEVELOPMENT.md to reference cargo make commands"
    priority: 5
    status: done
    notes: "Replace raw cargo commands with cargo make equivalents throughout"
  - id: T-011
    title: "Update CONTRIBUTING.md to reference cargo make commands"
    priority: 5
    status: done
    notes: "Replace cargo test / cargo clippy / cargo fmt with cargo make ci"
  - id: T-012
    title: "Update AGENTS.md if new patterns or workflows are introduced"
    priority: 5
    status: done
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

## 2026-02-07 — T-002 Completed
- **Task**: Create .config/nextest.toml
- **Status**: ✅ Done
- **Changes**:
  - `.config/nextest.toml` was already created during T-001 with the correct microralph pattern (slow-timeout period 5s, terminate-after 6)
  - Verified contents match `twitchax/microralph/.config/nextest.toml` exactly
  - No code changes needed; task was already satisfied
  - `cargo make uat` passes: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations.

## 2026-02-07 — T-003 In Progress
- **Task**: Add clippy denies to src/main.rs
- **Status**: ❌ Failed (UAT fails pending T-004)
- **Changes**:
  - Added clippy deny directives to `src/main.rs` matching the microralph pattern:
    - `#![deny(unused)]`
    - `#![deny(clippy::unwrap_used)]`
    - `#![deny(clippy::correctness)]`
    - `#![deny(clippy::complexity)]`
    - `#![deny(clippy::pedantic)]`
  - Added explanatory comments matching microralph style
  - `cargo make uat` fails: fmt-check ✅, clippy ❌ (47-49 deny violations across codebase)
  - Violations include: `unwrap_used`, `pedantic` lints (missing docs, needless borrows, etc.), `non_std_lazy_statics` (once_cell::sync::Lazy → std::sync::LazyLock), and others
  - These violations are expected and will be fixed in T-004 ("Fix all clippy deny violations across codebase")
- **Constitution Compliance**: No violations. The denies are correctly added; fixing the resulting violations is scoped to T-004.

## 2026-02-07 — T-003 + T-004 Completed
- **Task**: Add clippy denies to src/main.rs + Fix all clippy deny violations across codebase
- **Status**: ✅ Done
- **Changes**:
  - T-003 denies were already in place from previous attempt; T-004 fixes applied to make UAT pass
  - Replaced `once_cell::sync::Lazy` with `std::sync::LazyLock` across `src/game/shared/constants.rs` (9 instances)
  - Removed `once_cell` dependency from `Cargo.toml`
  - Added `#[must_use]` to 7 functions in `src/game/shared/helpers.rs` and 1 method in `src/game/shared/types.rs`
  - Added `#[allow(clippy::cast_possible_truncation)]` on 4 functions/blocks where `f64 as f32` is intentional (graphics conversions)
  - Replaced `as f64` casts with `f64::from()` for lossless `f32→f64` conversions in observer, player_clock, player_sprite, systems
  - Added `#[allow(clippy::needless_pass_by_value)]` on 7 Bevy system functions (Bevy requires pass-by-value for system params)
  - Replaced `.iter()`/`.iter_mut()` with `&query`/`&mut query` for idiomatic Bevy iteration (6 instances)
  - Fixed `uninlined_format_args` in observer clock text update
  - Fixed wildcard match in test to enumerate all variants explicitly
  - Replaced `_level` binding with `assert!(matches!(...))` to eliminate no-effect binding
  - `cargo make uat` passes: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations. Changes are mechanical lint fixes. `once_cell` removal is a dependency simplification justified by std library replacement.
- **Opportunistic UAT Verification**: uat-003 ("All clippy denies satisfied with zero warnings") is now verifiable — `cargo clippy --all-targets --all-features -- -D warnings` exits cleanly with zero errors.

## 2026-02-07 — T-005 Completed
- **Task**: Rewrite build.yml to use cargo-make and match microralph pattern
- **Status**: ✅ Done
- **Changes**:
  - Rewrote `.github/workflows/build.yml` to match microralph/kord CI pattern
  - Added `RUST_TOOLCHAIN: nightly-2025-12-22` env var for pinned toolchain
  - Replaced separate `test` and `clippy` jobs with single `test` job running `cargo make ci`
  - Added `rustfmt, clippy` components to the test job's toolchain setup
  - Added `cargo-bins/cargo-binstall@main` action and `cargo binstall cargo-make` across all jobs
  - Set `cache-bin: "false"` on all `Swatinem/rust-cache@v2` steps
  - Replaced `cargo install cargo-llvm-cov` with `cargo make codecov` in codecov job (uses binstall via Makefile.toml)
  - Replaced `cargo install cross` + `cross build` in Windows job with `cargo make build-windows`
  - Replaced raw `cargo build` in Linux and macOS jobs with `cargo make build-linux` / `cargo make build-macos`
  - Kept Bevy system dependencies (`libasound2-dev`, etc.) for Linux-based jobs
  - Kept `macos-15` runner (project's existing choice)
  - `cargo make uat` passes: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations. Changes are limited to the CI workflow file, matching established patterns from microralph/kord.

## 2026-02-07 — T-006 Completed
- **Task**: Update web.yml to use cargo-make and cargo-binstall for Trunk
- **Status**: ✅ Done
- **Changes**:
  - Updated `.github/workflows/web.yml` to match the established CI pattern from build.yml and microralph
  - Added `RUST_TOOLCHAIN: nightly-2025-12-22` env var for pinned toolchain
  - Replaced `cargo install trunk` with `cargo-bins/cargo-binstall@main` action + `cargo binstall cargo-make` + `cargo make build-web` (which uses binstall for trunk internally via Makefile.toml)
  - Added `cache-bin: "false"` to `Swatinem/rust-cache@v2` step
  - Pinned toolchain via `toolchain: ${{ env.RUST_TOOLCHAIN }}` in `dtolnay/rust-toolchain@nightly`
  - Makefile.toml already had `install-trunk` (cargo binstall) and `build-web` (trunk build --release) tasks from T-001
  - `cargo make uat` passes: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations. Changes are limited to the web CI workflow file, following the same pattern already established in build.yml.

## 2026-02-07 — T-007 Completed
- **Task**: Add copilot-setup-steps.yml workflow
- **Status**: ✅ Done
- **Changes**:
  - Created `.github/workflows/copilot-setup-steps.yml` modeled after kord's pattern
  - Includes: checkout, Bevy system dependencies (libasound2-dev, portaudio19-dev, build-essential, libpulse-dev, libdbus-1-dev, libudev-dev, libwayland-dev, libxkbcommon-dev), pinned nightly toolchain (nightly-2025-12-22) with rustfmt+clippy components, cargo-binstall, rust-cache with cache-all-crates, cargo tool installs (cargo-make, cargo-nextest, cargo-llvm-cov), and cargo fetch
  - Adapted from kord by: using actions/checkout@v4 (matching existing workflows), including full Bevy system deps (kord only needs libasound2-dev), installing relativity-relevant cargo tools instead of kord-specific ones (cargo-leptos, wasm-pack, etc.)
  - Workflow triggers on push/PR changes to itself, plus manual dispatch
  - `cargo make uat` passes: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations. New workflow file only, following established patterns.

## 2026-02-07 — T-008 Completed
- **Task**: Add codecov job to build.yml
- **Status**: ✅ Done
- **Changes**:
  - Verified that the codecov job already exists in `.github/workflows/build.yml` (lines 29-50), having been added during T-005 (build.yml rewrite)
  - The existing codecov job matches the microralph pattern exactly: `needs: test`, system deps, pinned toolchain, cargo-binstall, rust-cache with `cache-bin: "false"`, `cargo make codecov`, and `codecov/codecov-action@v5` with token and slug
  - The Makefile.toml `codecov` task (using `cargo llvm-cov nextest --lcov --output-path coverage.lcov`) was already created in T-001
  - No code changes needed; task was already satisfied by previous work
  - `cargo make uat` passes: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations. Minimal changes principle satisfied — no unnecessary modifications made since the work was already complete.

## 2026-02-07 — T-009 Completed
- **Task**: Add release and changelog tasks to Makefile.toml
- **Status**: ✅ Done
- **Changes**:
  - Created `cliff.toml` with git-cliff configuration matching microralph pattern (conventional commits, grouped by type with emoji headers, semantic versioning)
  - Updated `Makefile.toml` Publishing section to match microralph's release automation pattern:
    - Added `release-bump` task for standalone version bumping via cargo-release
    - Upgraded `release` task from simple `cargo release` command to full automated pipeline: CI → changelog → bump → commit → push → wait for CI artifacts → download artifacts
    - Enhanced `github-release` task with detailed logging, artifact staging with rename, fallback changelog text, and draft mode support (matching microralph's verbose style)
  - `changelog` task unchanged (already correct)
  - `cargo make uat` passes: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations. Changes are limited to the Publishing section of Makefile.toml and a new cliff.toml config file.

## 2026-02-07 — T-010 Completed
- **Task**: Update DEVELOPMENT.md to reference cargo make commands
- **Status**: ✅ Done
- **Changes**:
  - Replaced all raw `cargo` commands in `DEVELOPMENT.md` with `cargo make` equivalents:
    - `cargo build` → `cargo make build`
    - `cargo run --release` → `cargo make build-release`
    - `cargo fmt` → `cargo make fmt`
    - `cargo clippy --all-targets --all-features` → `cargo make clippy`
    - `cargo test` → `cargo make test`
    - Raw platform build commands → `cargo make build-linux`, `cargo make build-windows`, `cargo make build-macos`
  - Added new sections: Full CI Pipeline (`cargo make ci`), UAT (`cargo make uat`), Code Coverage (`cargo make codecov`/`cargo make codecov-html`), Changelog (`cargo make changelog`), Release (`cargo make release`), GitHub Release (`cargo make github-release`)
  - Added prerequisites for `cargo-make` and `cargo-binstall` in the Local Setup section
  - Updated Web Build section to use `cargo make build-web` instead of manual `cargo install trunk` + `trunk build`
  - Removed obsolete "Publishing to crates.io" section (project is a binary game, not a crate)
  - `cargo make uat` passes: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations. Changes are limited to documentation updates.

## 2026-02-07 — T-011 Completed
- **Task**: Update CONTRIBUTING.md to reference cargo make commands
- **Status**: ✅ Done
- **Changes**:
  - Replaced `cargo test` and `cargo clippy` with `cargo make ci` in step 3 of "How to Contribute"
  - Replaced `cargo fmt` with `cargo make fmt` in step 4 of "How to Contribute"
  - Replaced `cargo fmt` with `cargo make fmt` in Coding Guidelines
  - Replaced `cargo clippy` with `cargo make clippy` in Coding Guidelines
  - `cargo make uat` passes: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations. Changes are limited to documentation updates.

## 2026-02-07 — T-012 Completed
- **Task**: Update AGENTS.md if new patterns or workflows are introduced
- **Status**: ✅ Done
- **Changes**:
  - Updated Quick Start section: `cargo build` → `cargo make build`, added nextest note
  - Added new "Available `cargo make` Tasks" section with full table of all 18 tasks from Makefile.toml
  - Updated Conventions: clarified "never raw `cargo test`, `cargo clippy`" and added nextest/`.config/nextest.toml` reference
  - Added new "CI / Workflow Patterns" section documenting: pinned toolchain, cargo-binstall, copilot-setup-steps.yml, git-cliff/cliff.toml
  - `cargo make uat` passes: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations. Changes are limited to documentation updates.

## 2026-02-07 — uat-001 Verification
- **UAT**: cargo make ci passes cleanly (fmt-check, clippy with denies, nextest)
- **Status**: ✅ Verified
- **Method**: Existing test
- **Details**:
  - Ran `cargo make ci` which executes fmt-check, clippy with `-D warnings`, and nextest
  - fmt-check: ✅ passed
  - clippy: ✅ passed (zero warnings, all denies satisfied)
  - nextest: ✅ 2/2 tests passed (test_time_warp_level_enum_exists, test_spawn_level_handles_time_warp)
  - Exit code: 0 (clean pass)

## 2026-02-07 — uat-002 Verification
- **UAT**: cargo make uat passes cleanly
- **Status**: ✅ Verified
- **Method**: Existing test
- **Details**:
  - Ran `cargo make uat` which delegates to the `ci` task (fmt-check + clippy + nextest)
  - fmt-check: ✅ passed
  - clippy: ✅ passed (zero warnings, all denies satisfied)
  - nextest: ✅ 2/2 tests passed (test_time_warp_level_enum_exists, test_spawn_level_handles_time_warp)
  - Exit code: 0 (clean pass)

