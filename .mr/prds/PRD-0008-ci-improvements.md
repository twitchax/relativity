---
id: PRD-0008
title: "CI Improvements: Unified Pipeline, DRY Toolchain, and Mold Linker"
status: draft
owner: twitchax
created: 2026-02-08
updated: 2026-02-08
principles:
  - "Model CI after twitchax/microralph's build.yml pattern"
  - "Single build.yml for all CI jobs (test, codecov, platform builds, web deploy)"
  - "DRY toolchain pinning: derive from rust-toolchain.toml where possible"
  - "Use mold linker for faster Linux release builds"
  - "PRs and non-main pushes run test + codecov only; main runs full pipeline"
  - "Accept system dep repetition across Linux jobs for now"
references:
  - name: "microralph build.yml (reference CI)"
    url: "https://github.com/twitchax/microralph/blob/main/.github/workflows/build.yml"
  - name: "microralph Makefile.toml (reference tasks)"
    url: "https://github.com/twitchax/microralph/blob/main/Makefile.toml"
  - name: "Current relativity build.yml"
    url: "https://github.com/twitchax/relativity/blob/main/.github/workflows/build.yml"
  - name: "Current relativity web.yml"
    url: "https://github.com/twitchax/relativity/blob/main/.github/workflows/web.yml"
  - name: "peaceiris/actions-gh-pages"
    url: "https://github.com/peaceiris/actions-gh-pages"
acceptance_tests:
  - id: uat-001
    name: "CI passes on push to non-main branch (test + codecov only)"
    command: cargo make ci
    uat_status: unverified
  - id: uat-002
    name: "CI passes on push to main (full pipeline including platform builds and web deploy)"
    command: cargo make ci
    uat_status: unverified
  - id: uat-003
    name: "web.yml is removed; all jobs live in build.yml"
    command: "test ! -f .github/workflows/web.yml"
    uat_status: unverified
  - id: uat-004
    name: "Toolchain version is not hardcoded in workflow files (derived or centralized)"
    command: "! grep -q 'nightly-20' .github/workflows/build.yml"
    uat_status: unverified
  - id: uat-005
    name: "Linux release build uses mold linker"
    command: "grep -q mold .github/workflows/build.yml"
    uat_status: unverified
  - id: uat-006
    name: "Makefile.toml has release, github-release, and publish-all tasks"
    command: "grep -q 'tasks.release' Makefile.toml && grep -q 'tasks.github-release' Makefile.toml"
    uat_status: unverified
tasks:
  - id: T-001
    title: "Consolidate web.yml into build.yml"
    priority: 1
    status: todo
    notes: "Move the WASM/Trunk build and GitHub Pages deploy into build.yml as a new job gated on refs/heads/main. Delete web.yml."
  - id: T-002
    title: "DRY toolchain pinning in build.yml"
    priority: 1
    status: todo
    notes: "Remove hardcoded RUST_TOOLCHAIN env var. Read toolchain from rust-toolchain.toml or use a single env var at the top derived from the file."
  - id: T-003
    title: "Add mold linker to Linux release build"
    priority: 2
    status: todo
    notes: "Install clang + mold in the build_linux job. Set RUSTFLAGS to use mold. Match microralph's pattern."
  - id: T-004
    title: "Gate builds and deploys to main-only; PRs run test + codecov"
    priority: 1
    status: todo
    notes: "Change trigger to push + pull_request. Platform builds and web deploy use if: github.ref == 'refs/heads/main'. Test and codecov run on all pushes and PRs."
  - id: T-005
    title: "Update copilot-setup-steps.yml toolchain to match"
    priority: 3
    status: todo
    notes: "Remove hardcoded toolchain date from copilot-setup-steps.yml; use same DRY approach."
  - id: T-006
    title: "Add release, github-release, and publish-all tasks to Makefile.toml"
    priority: 2
    status: todo
    notes: "Port the release automation tasks from microralph's Makefile.toml, adapting repo name and removing wasm/OCI tasks that don't apply."
  - id: T-007
    title: "Modernize GitHub Pages deploy action"
    priority: 3
    status: todo
    notes: "Evaluate peaceiris/actions-gh-pages@v3 vs v4 or GitHub's built-in pages deployment. Upgrade if appropriate."
---

# Summary

Unify the CI pipeline into a single `build.yml`, DRY-up the toolchain pinning, add the mold linker for faster Linux builds, and port the release automation tasks from microralph. This brings relativity's CI in line with the established patterns in `twitchax/microralph`.

# Problem

The current CI setup has several issues:

1. **Two separate workflow files** (`build.yml` and `web.yml`) that could be a single pipeline, making it harder to reason about the full CI picture.
2. **Hardcoded toolchain dates** (`nightly-2025-12-22`) repeated in three workflow files, creating maintenance burden and drift risk.
3. **No mold linker** for Linux release builds, resulting in slower link times.
4. **Missing release automation tasks** in `Makefile.toml` — the release/publish workflow from microralph hasn't been ported.
5. **No PR-specific trigger** — the workflow only triggers on push, so PRs don't get CI feedback until after merge.

# Goals

1. Single `build.yml` containing all CI jobs: test, codecov, platform builds (Linux/Windows/macOS), web build + deploy.
2. Toolchain version derived from a single source, not hardcoded across files.
3. Mold linker for Linux release builds (matching microralph).
4. PR pushes trigger test + codecov; main pushes trigger full pipeline including builds and deploy.
5. Release automation tasks (`release`, `github-release`, `publish-all`) in Makefile.toml.
6. Modernized GitHub Pages deploy action.

# Technical Approach

## Unified build.yml Structure

```
on:
  push:
  pull_request:

jobs:
  test          → all pushes + PRs
  codecov       → all pushes + PRs (needs: test)
  build_linux   → main only (needs: test), uses mold
  build_windows → main only (needs: test)
  build_macos   → main only (needs: test)
  build_web     → main only (needs: test), Trunk build + GH Pages deploy
```

## DRY Toolchain

Use the `RUST_TOOLCHAIN` env var at the workflow level, but set it once — either:
- A single env var at the top of `build.yml` (simplest, matches microralph pattern), or
- A step that reads `rust-toolchain.toml` and exports the value.

Given microralph uses the env var pattern, we'll keep that for consistency but ensure it's the **only** place the version appears.

## Mold Linker

Add to `build_linux` job:
```yaml
- name: Install mold linker
  run: |
    sudo apt-get update
    sudo apt-get install -y clang mold
- run: cargo make build-linux
  env:
    RUSTFLAGS: "-C linker=clang -C link-arg=-fuse-ld=mold"
```

## Makefile.toml Release Tasks

Port from microralph, adapting:
- Repo name `relativity` instead of `microralph`
- Remove WASM/OCI tasks (not applicable — relativity uses Trunk)
- Keep `release`, `github-release`, `publish-all` patterns

# Assumptions

- The `CODECOV_TOKEN` secret is already configured for the repository.
- The `gh` CLI is available in the release task environment (local developer machine).
- `cargo-release` and `git-cliff` are installable via `cargo-binstall`.
- Trunk is already configured and working for WASM builds.

# Constraints

- System dependencies (libasound2-dev, etc.) must remain in each Linux job — no composite action for now.
- The `copilot-setup-steps.yml` workflow is a separate concern but should also have its toolchain updated.
- Bevy does not support `wasm32-wasip2`; only `wasm32-unknown-unknown` via Trunk applies.
- The GitHub Pages deploy must use the `gh-pages` branch approach (current setup).

# References to Code

- `.github/workflows/build.yml` — main CI workflow to rewrite
- `.github/workflows/web.yml` — to be merged into build.yml and deleted
- `.github/workflows/copilot-setup-steps.yml` — toolchain to update
- `Makefile.toml` — add release tasks
- `rust-toolchain.toml` — source of truth for toolchain channel
- `Trunk.toml` — WASM build configuration
- `index.html` — Trunk entry point

# Non-Goals (MVP)

- Composite actions or reusable workflows to DRY system deps across jobs
- Scheduled/nightly builds
- Windows test job (only build)
- Crates.io publishing (relativity is a game, not a library)
- `wasm32-wasip2` or OCI publishing

# History
