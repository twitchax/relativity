// E2E headless test: verifies that HUD label/header text entities use the
// display font (Orbitron) while numeric value readout entities use the
// monospace font (Hack Nerd Font Mono).
//
// This validates acceptance criterion uat-004: "Labels use a display font;
// numeric values use the monospace font".

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use common::{build_gameplay_app, enter_game};
use relativity::game::hud::{HudGravGamma, HudObserverTime, HudPlayerTime, HudSimRate, HudVelocityFraction, HudVelocityGamma};

/// Value readout entities must use the monospace font, and header/label
/// entities must use the display font. The two font handles must differ.
#[test]
fn labels_use_display_font_and_values_use_monospace_font() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // Load both font handles through the asset server (same paths as HUD code).
    let mono_handle: Handle<Font> = app.world().resource::<AssetServer>().load("fonts/HackNerdFontMono-Regular.ttf");
    let display_handle: Handle<Font> = app.world().resource::<AssetServer>().load("fonts/Orbitron-Regular.ttf");

    // The two fonts must be distinct assets.
    assert_ne!(mono_handle, display_handle, "monospace and display font handles must differ");

    // All six value readout entities should use the monospace font.
    let check_mono = |world: &mut World, name: &str, entity: Entity| {
        let text_font = world.get::<TextFont>(entity).unwrap_or_else(|| panic!("{name} should have TextFont"));
        assert_eq!(text_font.font, mono_handle, "{name} value readout should use the monospace font");
    };

    let tp = app.world_mut().query_filtered::<Entity, With<HudPlayerTime>>().single(app.world()).unwrap();
    check_mono(app.world_mut(), "HudPlayerTime", tp);

    let vg = app.world_mut().query_filtered::<Entity, With<HudVelocityGamma>>().single(app.world()).unwrap();
    check_mono(app.world_mut(), "HudVelocityGamma", vg);

    let gg = app.world_mut().query_filtered::<Entity, With<HudGravGamma>>().single(app.world()).unwrap();
    check_mono(app.world_mut(), "HudGravGamma", gg);

    let vf = app.world_mut().query_filtered::<Entity, With<HudVelocityFraction>>().single(app.world()).unwrap();
    check_mono(app.world_mut(), "HudVelocityFraction", vf);

    let to = app.world_mut().query_filtered::<Entity, With<HudObserverTime>>().single(app.world()).unwrap();
    check_mono(app.world_mut(), "HudObserverTime", to);

    let sr = app.world_mut().query_filtered::<Entity, With<HudSimRate>>().single(app.world()).unwrap();
    check_mono(app.world_mut(), "HudSimRate", sr);

    // At least one header/label text entity must use the display font.
    // "FLIGHT DATA" and "OBSERVER" are panel headers spawned with the display font.
    let mut found_display_font = false;
    for (text, text_font) in app.world_mut().query::<(&Text2d, &TextFont)>().iter(app.world()) {
        let content = text.as_str();
        if content == "FLIGHT DATA" || content == "OBSERVER" || content == "TIME" || content == "VELOCITY" || content == "GAMMA" || content == "RATE" {
            assert_eq!(text_font.font, display_handle, "label/header '{content}' should use the display font");
            found_display_font = true;
        }
    }

    assert!(found_display_font, "expected at least one header/label entity using the display font");
}
