// Unit tests for input handlers
//
// Moved from inline tests in src/state_machine/handlers.rs

use crate::config::Config;
use crate::state_machine::{AppState, handlers::ConfiguringHandler};
use crate::state_machine::handlers::InputHandler;
use crossterm::event::{KeyCode, KeyEvent};

#[test]
fn test_configuring_handler() {
    let handler = ConfiguringHandler;
    let state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
    };

    let events = handler.handle_key(
        KeyEvent::from(KeyCode::Enter),
        &state,
    );

    assert_eq!(events.len(), 1);
}
