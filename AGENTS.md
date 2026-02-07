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
cargo build

# Test
cargo make test

# Full CI (fmt, clippy, test)
cargo make ci

# UAT (the one true gate)
cargo make uat
```

## Conventions for Agents

- Keep changes minimal and focused; avoid unrelated refactors.
- Follow existing style; don't add license headers.
- Use `anyhow::Result` for fallible functions.
- Prefer `tracing` over `println!` for diagnostics.
- All dev commands route through `cargo make`.

### Code Style

- Use vertical whitespace generously to separate logical sections.
- Prefer explicitness over implicitness.
- Reduce nesting by using guard clauses and early returns.
- Prefer functional programming techniques where appropriate.

### Clippy Denies

Strict clippy denies are enforced in `src/main.rs`:
- `#![deny(unused)]`, `#![deny(clippy::unwrap_used)]`, `#![deny(clippy::correctness)]`, `#![deny(clippy::complexity)]`, `#![deny(clippy::pedantic)]`
- Bevy system functions require `#[allow(clippy::needless_pass_by_value)]` since Bevy's ECS requires pass-by-value for system parameters.
- Use `#[allow(clippy::cast_possible_truncation)]` on intentional `f64 as f32` conversions (e.g., graphics/pixel coordinates).
- Use `f64::from(f32_value)` instead of `f32_value as f64` for lossless casts.
- Use `std::sync::LazyLock` instead of `once_cell::sync::Lazy` for lazy statics.

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
