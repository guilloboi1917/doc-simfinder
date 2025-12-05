// Unit tests for AppState and StateMachine
//
// Moved from inline tests in src/state_machine/states.rs and mod.rs

use crate::config::Config;
use crate::state_machine::{AppState, StateMachine};

#[test]
fn test_state_machine_creation() {
    let config = Config::default();
    let initial_state = AppState::Configuring {
        config,
        validation_errors: vec![],
        walk_result: None,
    };
    let sm = StateMachine::new(initial_state);
    assert!(matches!(sm.current_state(), AppState::Configuring { .. }));
}

#[test]
fn test_state_config_access() {
    let config = Config::default();
    let state = AppState::Configuring {
        config: config.clone(),
        validation_errors: vec![],
        walk_result: None,
    };

    // Should be able to get config reference
    assert!(state.config().is_some());
    
    // Config values should match
    if let Some(cfg) = state.config() {
        assert_eq!(cfg.threshold, config.threshold);
    }
}
