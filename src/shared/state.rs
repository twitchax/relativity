use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Paused,
    Running,
    SimPaused,
    Finished,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    // AppState default is Menu.
    #[test]
    fn app_state_default_is_menu() {
        assert_eq!(AppState::default(), AppState::Menu);
    }

    // GameState default is Paused.
    #[test]
    fn game_state_default_is_paused() {
        assert_eq!(GameState::default(), GameState::Paused);
    }

    // AppState variants are distinct.
    #[test]
    fn app_state_variants_are_distinct() {
        assert_ne!(AppState::Menu, AppState::InGame);
    }

    // GameState variants are distinct.
    #[test]
    fn game_state_variants_are_distinct() {
        assert_ne!(GameState::Paused, GameState::Running);
        assert_ne!(GameState::Running, GameState::Finished);
        assert_ne!(GameState::Paused, GameState::Finished);
        assert_ne!(GameState::Failed, GameState::Paused);
        assert_ne!(GameState::Failed, GameState::Running);
        assert_ne!(GameState::Failed, GameState::Finished);
        assert_ne!(GameState::SimPaused, GameState::Paused);
        assert_ne!(GameState::SimPaused, GameState::Running);
        assert_ne!(GameState::SimPaused, GameState::Finished);
        assert_ne!(GameState::SimPaused, GameState::Failed);
    }

    // AppState Debug produces expected strings.
    #[test]
    fn app_state_debug_format() {
        assert_eq!(format!("{:?}", AppState::Menu), "Menu");
        assert_eq!(format!("{:?}", AppState::InGame), "InGame");
    }

    // GameState Debug produces expected strings.
    #[test]
    fn game_state_debug_format() {
        assert_eq!(format!("{:?}", GameState::Paused), "Paused");
        assert_eq!(format!("{:?}", GameState::Running), "Running");
        assert_eq!(format!("{:?}", GameState::SimPaused), "SimPaused");
        assert_eq!(format!("{:?}", GameState::Finished), "Finished");
        assert_eq!(format!("{:?}", GameState::Failed), "Failed");
    }

    // AppState Clone produces an equal value.
    #[test]
    fn app_state_clone() {
        let state = AppState::InGame;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    // GameState Clone produces an equal value.
    #[test]
    fn game_state_clone() {
        let state = GameState::Running;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    // AppState Hash is consistent for equal values.
    #[test]
    fn app_state_hash_consistency() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        AppState::Menu.hash(&mut h1);
        AppState::Menu.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    // GameState Hash is consistent for equal values.
    #[test]
    fn game_state_hash_consistency() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        GameState::Finished.hash(&mut h1);
        GameState::Finished.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }
}
