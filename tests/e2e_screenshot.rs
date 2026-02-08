// E2E headless screenshot test: verifies the visual output of the game against
// baseline images.
//
// Uses the full DefaultPlugins (minus WinitPlugin) with an offscreen render
// target. After the scene is rendered, a screenshot is captured via Bevy's
// `Screenshot` component and compared pixel-by-pixel against a committed
// baseline PNG in `tests/baselines/`.
//
// On first run (no baseline exists), the test saves the rendered image as the
// new baseline and fails with a review prompt. Commit the baseline and re-run.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use std::path::PathBuf;

use bevy::prelude::*;
use common::{
    headless::{build_headless_render_app, suppress_backtrace, wait_for_plugins},
    screenshot::{assert_screenshot_matches, capture_screenshot_to_file},
};
use relativity::shared::state::AppState;

/// Temporary directory for screenshot captures (cleaned up after comparison).
fn temp_capture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("tmp").join("screenshots")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Capture a screenshot of Level 1 after spawning and compare against baseline.
///
/// This verifies that:
/// 1. The headless render pipeline produces an image.
/// 2. The rendered scene is visually stable across code changes.
///
/// The first time this runs, it creates the baseline image and fails with a
/// review prompt. Commit the baseline PNG in `tests/baselines/` and re-run.
#[test]
fn level1_spawn_screenshot_matches_baseline() {
    suppress_backtrace();

    let mut app = build_headless_render_app();
    wait_for_plugins(&mut app);

    // Transition into the game so the level is spawned and rendered.
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::InGame);

    // Run enough frames for the scene to stabilize (spawn + render).
    for _ in 0..100 {
        app.update();
    }

    // Capture the screenshot to a temp file.
    let capture_dir = temp_capture_dir();
    std::fs::create_dir_all(&capture_dir).expect("failed to create temp capture directory");
    let capture_path = capture_dir.join("level1_spawn.png");

    // Remove any stale capture from a previous run.
    std::fs::remove_file(&capture_path).ok();

    capture_screenshot_to_file(&mut app, &capture_path, 120);

    // Compare against the baseline.
    assert_screenshot_matches("level1_spawn", &capture_path);
}
