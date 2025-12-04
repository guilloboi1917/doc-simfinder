// Focus management system for TUI
//
// See docs/copilot/tui-integration.md for focus patterns

use crate::state_machine::AppState;

/// Focusable elements in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Focus {
    // Configuration state
    PathInput,
    QueryInput,
    OptionsPanel,
    StartButton,

    // Results state
    FileList,
    FilePreview,
    StatsPanel,
    ActionPanel,

    // File detail state
    FileContent,
    NavigationButtons,

    // Global
    Help,
}

/// Direction for focus movement
#[derive(Debug, Clone, Copy)]
pub enum FocusDirection {
    Next,
    Previous,
    To(Focus),
}

/// Manages focus state and transitions
pub struct FocusManager {
    current: Focus,
    focus_history: Vec<Focus>,
    available_targets: Vec<Focus>,
}

impl FocusManager {
    /// Create a new focus manager for the given state
    pub fn new_for_state(state: &AppState) -> Self {
        let available_targets = match state {
            AppState::Configuring { .. } => vec![
                Focus::PathInput,
                Focus::QueryInput,
                Focus::OptionsPanel,
                Focus::StartButton,
            ],
            AppState::ViewingResults { .. } => vec![
                Focus::FileList,
                Focus::FilePreview,
            ],
            AppState::ViewingFileDetail { .. } => vec![
                Focus::FileContent,
                Focus::NavigationButtons,
            ],
            _ => vec![],
        };

        let current = available_targets.first().copied().unwrap_or(Focus::FileList);

        Self {
            current,
            focus_history: Vec::new(),
            available_targets,
        }
    }

    /// Get the current focus
    pub fn current(&self) -> Focus {
        self.current
    }

    /// Move focus in the specified direction
    pub fn move_focus(&mut self, direction: FocusDirection) {
        if self.available_targets.is_empty() {
            return;
        }

        let current_index = self
            .available_targets
            .iter()
            .position(|&f| f == self.current)
            .unwrap_or(0);

        let new_focus = match direction {
            FocusDirection::Next => {
                let next_index = (current_index + 1) % self.available_targets.len();
                self.available_targets[next_index]
            }
            FocusDirection::Previous => {
                let prev_index = if current_index == 0 {
                    self.available_targets.len() - 1
                } else {
                    current_index - 1
                };
                self.available_targets[prev_index]
            }
            FocusDirection::To(target) => {
                if self.available_targets.contains(&target) {
                    target
                } else {
                    return; // Invalid target, don't change focus
                }
            }
        };

        self.focus_history.push(self.current);
        self.current = new_focus;
    }

    /// Go back to previous focus
    pub fn go_back(&mut self) {
        if let Some(previous) = self.focus_history.pop() {
            self.current = previous;
        }
    }

    /// Check if a specific element has focus
    pub fn is_focused(&self, focus: Focus) -> bool {
        self.current == focus
    }

    /// Get all available focus targets for current state
    pub fn available_targets(&self) -> &[Focus] {
        &self.available_targets
    }
}
