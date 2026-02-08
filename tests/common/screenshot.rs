#![allow(dead_code)]

//! Screenshot comparison utilities for visual regression testing.
//!
//! Provides helpers to capture a rendered frame from a headless Bevy app and
//! compare it against a baseline PNG image.
//!
//! ## Baseline workflow
//!
//! 1. On the first run, no baseline exists — the test saves the rendered image
//!    as the new baseline and panics with a review prompt.
//! 2. Review and commit the baseline PNG in `tests/baselines/`.
//! 3. On subsequent runs the test compares the rendered image against the
//!    committed baseline using per-pixel RMSE.
//! 4. If the images diverge beyond the threshold, the test fails and writes
//!    `<name>.actual.png` and `<name>.diff.png` next to the baseline for
//!    visual inspection.

use std::path::{Path, PathBuf};

use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};

use super::headless::OffscreenRenderTarget;

/// Default per-channel RMSE threshold (0–255 scale).
const DEFAULT_THRESHOLD: f64 = 2.0;

// ------------------------------------------------------------------
// Screenshot capture
// ------------------------------------------------------------------

/// Capture the current offscreen render target to a PNG file on disk.
///
/// Spawns a Bevy `Screenshot` entity targeting the offscreen image and
/// runs update frames until the file has been written.
///
/// # Panics
///
/// Panics if the screenshot is not saved within `max_frames` updates.
pub fn capture_screenshot_to_file(app: &mut App, path: &Path, max_frames: usize) {
    let render_target = app.world().resource::<OffscreenRenderTarget>();
    let handle = render_target.handle.clone();
    let owned_path = path.to_path_buf();

    app.world_mut().spawn(Screenshot::image(handle)).observe(save_to_disk(owned_path));

    for _ in 0..max_frames {
        app.update();

        if path.exists() {
            return;
        }
    }

    panic!("Screenshot was not saved to {} within {max_frames} frames", path.display());
}

// ------------------------------------------------------------------
// Baseline comparison
// ------------------------------------------------------------------

/// Directory where baseline screenshots live.
fn baselines_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("baselines")
}

/// Assert that a screenshot matches its baseline within the default RMSE
/// threshold.
///
/// * `name` — logical name used to derive file paths (e.g. `"level1_spawn"`).
/// * `actual_path` — path to the just-captured PNG screenshot.
///
/// If no baseline exists yet, the actual screenshot is promoted to baseline
/// and the test panics with a review prompt.
pub fn assert_screenshot_matches(name: &str, actual_path: &Path) {
    assert_screenshot_matches_with_threshold(name, actual_path, DEFAULT_THRESHOLD);
}

/// Like [`assert_screenshot_matches`], but with a custom RMSE threshold.
pub fn assert_screenshot_matches_with_threshold(name: &str, actual_path: &Path, threshold: f64) {
    let baseline_path = baselines_dir().join(format!("{name}.png"));
    let diff_path = baselines_dir().join(format!("{name}.diff.png"));
    let actual_dest = baselines_dir().join(format!("{name}.actual.png"));

    // Load the just-captured image.
    let actual = image::open(actual_path).unwrap_or_else(|e| panic!("failed to open actual screenshot {}: {e}", actual_path.display()));
    let actual_rgba = actual.to_rgba8();

    // If no baseline exists, promote the actual image and ask for review.
    if !baseline_path.exists() {
        std::fs::create_dir_all(baselines_dir()).expect("failed to create baselines directory");

        actual_rgba
            .save(&baseline_path)
            .unwrap_or_else(|e| panic!("failed to save new baseline {}: {e}", baseline_path.display()));

        // Clean up the temp capture.
        std::fs::remove_file(actual_path).ok();

        panic!(
            "No baseline found for '{name}'.\n\
             Created new baseline at {}.\n\
             Review and commit it, then re-run the test.",
            baseline_path.display()
        );
    }

    // Load the baseline.
    let baseline = image::open(&baseline_path).unwrap_or_else(|e| panic!("failed to open baseline {}: {e}", baseline_path.display()));
    let baseline_rgba = baseline.to_rgba8();

    // Dimension check.
    if actual_rgba.dimensions() != baseline_rgba.dimensions() {
        // Move the actual image next to the baseline for inspection.
        std::fs::copy(actual_path, &actual_dest).ok();
        std::fs::remove_file(actual_path).ok();

        panic!(
            "Screenshot '{name}' dimensions mismatch.\n\
             Baseline: {}×{}\n\
             Actual:   {}×{}\n\
             Delete the baseline to regenerate.",
            baseline_rgba.width(),
            baseline_rgba.height(),
            actual_rgba.width(),
            actual_rgba.height(),
        );
    }

    // Pixel comparison (RMSE over all channels).
    let rmse = compute_rmse(actual_rgba.as_raw(), baseline_rgba.as_raw());

    if rmse > threshold {
        // Save artefacts for review.
        std::fs::copy(actual_path, &actual_dest).ok();

        let diff_data = generate_diff_image(actual_rgba.as_raw(), baseline_rgba.as_raw(), actual_rgba.width(), actual_rgba.height());
        save_rgba_png(&diff_data, actual_rgba.width(), actual_rgba.height(), &diff_path);

        std::fs::remove_file(actual_path).ok();

        panic!(
            "Screenshot '{name}' does not match baseline.\n\
             RMSE: {rmse:.4} (threshold: {threshold:.4})\n\
             Baseline: {}\n\
             Actual:   {}\n\
             Diff:     {}\n\
             Delete the baseline to regenerate.",
            baseline_path.display(),
            actual_dest.display(),
            diff_path.display(),
        );
    }

    // Test passed — clean up temp files.
    std::fs::remove_file(actual_path).ok();
    std::fs::remove_file(&diff_path).ok();
    // Don't remove actual_dest since it shouldn't exist at this point.
}

// ------------------------------------------------------------------
// Pixel math
// ------------------------------------------------------------------

/// Root-mean-square error across all channels (RGBA).
fn compute_rmse(actual: &[u8], baseline: &[u8]) -> f64 {
    assert_eq!(actual.len(), baseline.len(), "image data length mismatch");

    let n = actual.len() as f64;

    let total_sq_diff: f64 = actual
        .iter()
        .zip(baseline.iter())
        .map(|(&a, &b)| {
            let diff = f64::from(a) - f64::from(b);
            diff * diff
        })
        .sum();

    (total_sq_diff / n).sqrt()
}

/// Produce a red-channel diff image amplifying pixel differences.
fn generate_diff_image(actual: &[u8], baseline: &[u8], width: u32, height: u32) -> Vec<u8> {
    let pixel_count = (width * height) as usize;
    let mut diff = vec![0u8; pixel_count * 4];

    for i in 0..pixel_count {
        let offset = i * 4;

        let r_diff = (f64::from(actual[offset]) - f64::from(baseline[offset])).abs();
        let g_diff = (f64::from(actual[offset + 1]) - f64::from(baseline[offset + 1])).abs();
        let b_diff = (f64::from(actual[offset + 2]) - f64::from(baseline[offset + 2])).abs();

        // Amplify differences to make them clearly visible.
        let max_diff = r_diff.max(g_diff).max(b_diff);

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let intensity = (max_diff * 10.0).min(255.0) as u8;

        diff[offset] = intensity; // R
        diff[offset + 1] = 0; // G
        diff[offset + 2] = 0; // B
        diff[offset + 3] = 255; // A
    }

    diff
}

/// Save raw RGBA pixel data as a PNG file.
fn save_rgba_png(data: &[u8], width: u32, height: u32, path: &Path) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create parent directory for PNG");
    }

    let img: image::RgbaImage = image::ImageBuffer::from_raw(width, height, data.to_vec()).expect("failed to create image buffer from raw pixel data");

    img.save(path).unwrap_or_else(|e| panic!("failed to save PNG to {}: {e}", path.display()));
}
