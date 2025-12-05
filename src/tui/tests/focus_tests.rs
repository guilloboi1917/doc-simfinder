// Unit tests for TUI Focus Manager
//
// Moved from inline tests in src/tui/focus.rs for better organization

use crate::config::Config;
use crate::state_machine::AppState;
use crate::tui::focus::{Focus, FocusDirection, FocusManager};

#[test]
fn test_focus_manager_creation() {
    let state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
        walk_result: None,
    };
    let fm = FocusManager::new_for_state(&state);
    assert_eq!(fm.current(), Focus::PathInput);
}

#[test]
fn test_focus_navigation() {
    let state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
        walk_result: None,
    };
    let mut fm = FocusManager::new_for_state(&state);

    fm.move_focus(FocusDirection::Next);
    assert_eq!(fm.current(), Focus::QueryInput);

    fm.move_focus(FocusDirection::Previous);
    assert_eq!(fm.current(), Focus::PathInput);
}

#[test]
fn test_focus_wrapping() {
    let state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
        walk_result: None,
    };
    let mut fm = FocusManager::new_for_state(&state);

    // Configuring state has 5 elements: PathInput, QueryInput, FileList, OptionsPanel, StartButton
    // Starting at PathInput (0), move through all elements
    fm.move_focus(FocusDirection::Next);
    assert_eq!(fm.current(), Focus::QueryInput);
    
    fm.move_focus(FocusDirection::Next);
    assert_eq!(fm.current(), Focus::FileList);
    
    fm.move_focus(FocusDirection::Next);
    assert_eq!(fm.current(), Focus::OptionsPanel);
    
    fm.move_focus(FocusDirection::Next);
    assert_eq!(fm.current(), Focus::StartButton);
    
    // Should wrap around to beginning
    fm.move_focus(FocusDirection::Next);
    assert_eq!(fm.current(), Focus::PathInput);
}
