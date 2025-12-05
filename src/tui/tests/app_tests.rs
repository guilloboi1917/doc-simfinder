// Unit tests for TUI App
//
// Moved from inline tests in src/tui/app.rs for better organization

use crate::config::Config;
use crate::state_machine::AppState;
use crate::tui::{App, focus::FocusDirection};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

#[test]
fn test_app_creation() {
    let initial_state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
        walk_result: None,
        autocomplete_available: false,
        autocomplete_suggestion: None,
    };
    let app = App::new(initial_state);
    assert!(!app.should_quit);
}

#[tokio::test]
async fn test_quit_handling() {
    let initial_state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
        walk_result: None,
        autocomplete_available: false,
        autocomplete_suggestion: None,
    };
    let mut app = App::new(initial_state);

    // Simulate Ctrl+Q quit key (use modifiers)
    let mut quit_key = KeyEvent::from(KeyCode::Char('q'));
    quit_key.modifiers = crossterm::event::KeyModifiers::CONTROL;
    app.handle_key(quit_key);

    // Should trigger quit
    assert!(app.should_quit || matches!(app.current_state(), AppState::Exiting));
}

#[tokio::test]
async fn test_character_input_no_duplication() {
    let initial_state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
        walk_result: None,
        autocomplete_available: false,
        autocomplete_suggestion: None,
    };
    let mut app = App::new(initial_state);

    // Focus should start on PathInput
    assert_eq!(app.focus_manager.current(), crate::tui::focus::Focus::PathInput);

    // Type a single character
    let slash_key = KeyEvent::from(KeyCode::Char('/'));
    app.handle_key(slash_key);

    // Config should have exactly one character
    if let Some(config) = app.current_state().config() {
        assert_eq!(config.search_path.to_string_lossy(), "/");
    } else {
        panic!("Expected Configuring state with config");
    }
}

#[test]
fn test_query_input() {
    let initial_state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
        walk_result: None,
        autocomplete_available: false,
        autocomplete_suggestion: None,
    };
    let mut app = App::new(initial_state);

    // Move focus to QueryInput
    app.focus_manager.move_focus(FocusDirection::Next);
    assert_eq!(app.focus_manager.current(), crate::tui::focus::Focus::QueryInput);

    // Type characters
    app.handle_key(KeyEvent::from(KeyCode::Char('t')));
    app.handle_key(KeyEvent::from(KeyCode::Char('e')));
    app.handle_key(KeyEvent::from(KeyCode::Char('s')));
    app.handle_key(KeyEvent::from(KeyCode::Char('t')));

    // Should have the typed string
    if let Some(config) = app.current_state().config() {
        assert_eq!(config.query, "test");
    } else {
        panic!("Expected Configuring state with config");
    }
}

#[tokio::test]
async fn test_backspace_in_input() {
    let initial_state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
        walk_result: None,
        autocomplete_available: false,
        autocomplete_suggestion: None,
    };
    let mut app = App::new(initial_state);

    // Type some characters in PathInput
    app.handle_key(KeyEvent::from(KeyCode::Char('a')));
    app.handle_key(KeyEvent::from(KeyCode::Char('b')));
    app.handle_key(KeyEvent::from(KeyCode::Char('c')));
    
    if let Some(config) = app.current_state().config() {
        assert_eq!(config.search_path.to_string_lossy(), "abc");
    }

    // Backspace once
    app.handle_key(KeyEvent::from(KeyCode::Backspace));
    
    if let Some(config) = app.current_state().config() {
        assert_eq!(config.search_path.to_string_lossy(), "ab");
    }
}

#[tokio::test]
async fn test_key_release_events_ignored() {
    let initial_state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
        walk_result: None,
        autocomplete_available: false,
        autocomplete_suggestion: None,
    };
    let mut app = App::new(initial_state);

    // Create a key release event (not a press)
    let mut release_event = KeyEvent::from(KeyCode::Char('x'));
    release_event.kind = KeyEventKind::Release;
    
    app.handle_key(release_event);
    
    // Config should still be empty - release event should be ignored
    if let Some(config) = app.current_state().config() {
        assert_eq!(config.search_path.to_string_lossy(), "");
    }
    
    // Now send a press event
    let press_event = KeyEvent::from(KeyCode::Char('x'));
    app.handle_key(press_event);
    
    // Config should now have the character
    if let Some(config) = app.current_state().config() {
        assert_eq!(config.search_path.to_string_lossy(), "x");
    }
}
