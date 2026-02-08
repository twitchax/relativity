// E2E headless test: verifies the fade overlay animates on state transitions.
//
// Triggers a fade-out via `FadeState::start_fade_out`, runs the app frame-by-frame,
// and asserts:
//   1. FadeState transitions FadeOut → state change → FadeIn → inactive.
//   2. BackgroundColor alpha on the FadeOverlay entity interpolates 0 → 1 → 0.
//   3. Mid-fade alpha is approximately 0.5.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use relativity::{
    game::fade::{FadeDirection, FadeOverlay, FadeState},
    shared::state::{AppState, GameState},
};

/// Read the overlay's current alpha value.
fn overlay_alpha(app: &mut App) -> f32 {
    let bg = app
        .world_mut()
        .query_filtered::<&BackgroundColor, With<FadeOverlay>>()
        .single(app.world())
        .expect("FadeOverlay entity should exist");

    bg.0.alpha()
}

/// Read the current FadeDirection (clone).
fn fade_direction(app: &App) -> Option<FadeDirection> {
    app.world().resource::<FadeState>().active.clone()
}

#[test]
fn fade_overlay_animates_through_full_cycle() {
    let mut app = common::build_gameplay_app();

    // Enter InGame so the game plugin is fully active.
    common::enter_game(&mut app);
    assert_eq!(app.world().resource::<State<AppState>>().get().clone(), AppState::InGame);

    // Verify the overlay starts fully transparent and no fade is active.
    let initial_alpha = overlay_alpha(&mut app);
    assert!(initial_alpha < 0.01, "overlay should start transparent, got {initial_alpha}");
    assert!(fade_direction(&app).is_none(), "no fade should be active initially");

    // Start a fade-out transition back to Menu.
    app.world_mut().resource_mut::<FadeState>().start_fade_out(AppState::Menu, GameState::Paused);

    assert!(matches!(fade_direction(&app), Some(FadeDirection::Out { .. })), "FadeState should be FadeOut after start_fade_out");

    // --- Phase 1: Fade-out (alpha 0 → 1 over ~18 frames at 60fps = 0.3s) ---

    // Run roughly half the fade-out (~9 frames) and check mid-fade alpha.
    for _ in 0..9 {
        app.update();
    }

    let mid_fade_out_alpha = overlay_alpha(&mut app);
    assert!(mid_fade_out_alpha > 0.3 && mid_fade_out_alpha < 0.7, "mid-fade-out alpha should be ~0.5, got {mid_fade_out_alpha}");
    assert!(matches!(fade_direction(&app), Some(FadeDirection::Out { .. })), "should still be fading out at mid-point");

    // Run remaining frames to complete fade-out (+ small buffer).
    for _ in 0..12 {
        app.update();
    }

    // After fade-out completes, the system should have:
    // 1. Applied the state transition (AppState → Menu).
    // 2. Started the fade-in automatically.
    assert!(
        matches!(fade_direction(&app), Some(FadeDirection::In)),
        "FadeState should transition to FadeIn after fade-out completes, got {:?}",
        fade_direction(&app)
    );

    // --- Phase 2: Fade-in (alpha 1 → 0 over ~18 frames) ---

    // Run roughly half the fade-in (~9 frames).
    for _ in 0..9 {
        app.update();
    }

    let mid_fade_in_alpha = overlay_alpha(&mut app);
    assert!(mid_fade_in_alpha > 0.3 && mid_fade_in_alpha < 0.7, "mid-fade-in alpha should be ~0.5, got {mid_fade_in_alpha}");

    // Complete the fade-in (+ buffer).
    for _ in 0..12 {
        app.update();
    }

    // Fade should now be fully complete.
    assert!(fade_direction(&app).is_none(), "fade should be inactive after full cycle, got {:?}", fade_direction(&app));

    let final_alpha = overlay_alpha(&mut app);
    assert!(final_alpha < 0.01, "overlay should be fully transparent after fade-in, got {final_alpha}");

    // Verify the state transition actually happened.
    let final_app_state = app.world().resource::<State<AppState>>().get().clone();
    assert_eq!(final_app_state, AppState::Menu, "AppState should have transitioned to Menu via the fade");
}
