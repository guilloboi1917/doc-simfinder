// Layout management for TUI
//
// See docs/copilot/tui-integration.md for layout patterns

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use crate::state_machine::AppState;

/// Pane identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaneId {
    PathInput,
    QueryInput,
    OptionsPanel,
    StartButton,
    FileList,
    FilePreview,
    StatsPanel,
    ActionPanel,
    FileContent,
    NavigationButtons,
    ErrorMessage,
}

/// Layout configuration for different states
pub struct LayoutConfig {
    pub main_direction: Direction,
    pub main_constraints: Vec<Constraint>,
}

impl LayoutConfig {
    /// Create layout configuration for a given state
    pub fn for_state(state: &AppState) -> Self {
        match state {
            AppState::Configuring { .. } => Self::configuring_layout(),
            AppState::ViewingResults { .. } => Self::results_layout(),
            AppState::ViewingFileDetail { .. } => Self::file_detail_layout(),
            AppState::Error { .. } => Self::error_layout(),
            _ => Self::default_layout(),
        }
    }

    fn configuring_layout() -> Self {
        Self {
            main_direction: Direction::Vertical,
            main_constraints: vec![
                Constraint::Length(3),  // Path input
                Constraint::Length(3),  // Query input
                Constraint::Min(8),    // Found files
                Constraint::Length(7),  // Options
                Constraint::Length(3),  // Start button
            ],
        }
    }

    fn results_layout() -> Self {
        Self {
            main_direction: Direction::Horizontal,
            main_constraints: vec![
                Constraint::Percentage(60),  // File list
                Constraint::Percentage(40),  // Preview + stats + actions
            ],
        }
    }

    fn file_detail_layout() -> Self {
        Self {
            main_direction: Direction::Vertical,
            main_constraints: vec![
                Constraint::Min(10),    // File content
                Constraint::Length(3),  // Navigation
            ],
        }
    }

    fn error_layout() -> Self {
        Self {
            main_direction: Direction::Vertical,
            main_constraints: vec![
                Constraint::Min(5),     // Error message
                Constraint::Length(3),  // Actions
            ],
        }
    }

    fn default_layout() -> Self {
        Self {
            main_direction: Direction::Vertical,
            main_constraints: vec![Constraint::Percentage(100)],
        }
    }

    /// Split an area according to this layout
    pub fn split(&self, area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(self.main_direction)
            .constraints(self.main_constraints.clone())
            .split(area)
            .to_vec()
    }
}

/// Create a two-column split for results view
pub fn results_two_column(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    (chunks[0], chunks[1])
}

/// Create a right panel split (preview, stats, actions)
pub fn right_panel_split(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),  // Preview
            Constraint::Length(5),       // Stats
            Constraint::Min(3),          // Actions
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2])
}
