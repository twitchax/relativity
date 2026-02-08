// E2E headless test: verifies that the menu screen spawns a Button node for
// every `CurrentLevel` variant, each with the correct display-name text.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use relativity::game::levels::CurrentLevel;

#[test]
fn menu_spawns_button_for_each_level_variant() {
    let mut app = common::build_gameplay_app();

    // Add MenuPlugin — gameplay helper omits it because most tests skip the menu.
    app.add_plugins(relativity::menu::MenuPlugin);

    // Default AppState is Menu, so the first update triggers OnEnter(Menu) → spawn_menu.
    app.update();

    // Collect expected display names from all CurrentLevel variants.
    let expected_names: Vec<String> = CurrentLevel::all().iter().map(ToString::to_string).collect();

    // Query all Button entities.
    let button_count = app.world_mut().query_filtered::<Entity, With<Button>>().iter(app.world()).count();

    assert_eq!(
        button_count,
        expected_names.len(),
        "Expected one Button per CurrentLevel variant ({} variants), but found {}",
        expected_names.len(),
        button_count,
    );

    // Verify each expected name appears as a Text child of a Button.
    for name in &expected_names {
        let found = app.world_mut().query::<&Text>().iter(app.world()).any(|text| text.0 == *name);

        assert!(found, "Expected a Text node with content \"{name}\" for a level button");
    }
}
