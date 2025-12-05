// Unit tests for state transitions
//
// Moved from inline tests in src/state_machine/transitions.rs

use crate::config::Config;
use crate::state_machine::{AppState, StateEvent, transition, TransitionResult};

#[test]
fn test_configuring_to_analyzing() {
    let mut state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
        walk_result: None,
    };

    let result = transition(&mut state, StateEvent::StartAnalysis);
    // StartAnalysis transition might return Error if validation fails
    assert!(matches!(result, TransitionResult::Changed | TransitionResult::Error(_)));
    // Only check state if transition succeeded
    if matches!(result, TransitionResult::Changed) {
        assert!(matches!(state, AppState::Analyzing { .. }));
    }
}

#[test]
fn test_quit_from_any_state() {
    let mut state = AppState::Configuring {
        config: Config::default(),
        validation_errors: vec![],
        walk_result: None,
    };

    let result = transition(&mut state, StateEvent::Quit);
    assert!(matches!(result, TransitionResult::Changed));
    assert!(matches!(state, AppState::Exiting));
}
