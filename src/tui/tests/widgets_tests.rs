// Unit tests for TUI Widgets
//
// Moved from inline tests in src/tui/widgets.rs for better organization

use crate::config::Config;
use crate::state_machine::AppState;
use crate::tui::widgets::Dashboard;

#[test]
fn test_dashboard_creation() {
    let config = Config::default();
    let state = AppState::Configuring {
        config,
        validation_errors: vec![],
    };
    let _dashboard = Dashboard::new_for_state(&state);
    // Test that dashboard was created without panicking
}
