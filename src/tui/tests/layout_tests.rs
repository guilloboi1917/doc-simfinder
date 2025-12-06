// Unit tests for TUI Layout
//
// Moved from inline tests in src/tui/layout.rs for better organization

use crate::config::Config;
use crate::state_machine::AppState;
use crate::tui::layout::LayoutConfig;
use ratatui::layout::Direction;

#[test]
fn test_layout_creation() {
    let config = Config::default();
    let state = AppState::Configuring {
        config,
        validation_errors: vec![],
        walk_result: None,
        autocomplete_available: false,
        autocomplete_suggestion: None,
    };

    let layout = LayoutConfig::for_state(&state);
    assert_eq!(layout.main_direction, Direction::Vertical);
}
