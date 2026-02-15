// Headless render smoke test: verifies the game bootstraps and the render graph
// executes without a display server or GPU window.
//
// Uses DefaultPlugins with WinitPlugin disabled and no primary window.
// A camera renders to an offscreen Image target so the render graph actually
// executes. We drive the app manually via `app.update()` instead of `app.run()`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use common::headless::{build_headless_render_app, suppress_backtrace, wait_for_plugins};

#[test]
#[ignore]
fn headless_app_boots_and_runs_without_panic() {
    // cargo-make sets RUST_BACKTRACE=full by default. Any non-zero value causes
    // wgpu/Bevy's Metal render graph to hang on macOS (backtrace symbolication
    // contends with GPU driver internals). Disable it before Bevy initializes.
    suppress_backtrace();

    let mut app = build_headless_render_app();

    // Complete plugin initialization before calling update().
    // The render plugin initializes the GPU asynchronously; finish() and
    // cleanup() ensure all plugin lifecycle hooks have run.
    wait_for_plugins(&mut app);

    // Run several update cycles — the primary goal is to prove the app
    // bootstraps and the render graph executes without panicking.
    for _ in 0..5 {
        app.update();
    }
}
