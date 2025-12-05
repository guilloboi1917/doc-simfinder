// Input handlers per state
//
// See docs/copilot/state-machine.md for input handling patterns

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use super::{AppState, StateEvent};

/// Trait for handling input in a specific state
pub trait InputHandler {
    /// Handle a key event and return resulting state events
    fn handle_key(&self, key: KeyEvent, state: &AppState) -> Vec<StateEvent>;
}

/// Get the appropriate input handler for a state
pub fn get_handler_for_state(state: &AppState) -> Box<dyn InputHandler> {
    match state {
        AppState::Configuring { .. } => Box::new(ConfiguringHandler),
        AppState::Analyzing { .. } => Box::new(AnalyzingHandler),
        AppState::ViewingResults { .. } => Box::new(ResultsHandler),
        AppState::ViewingFileDetail { .. } => Box::new(FileDetailHandler),
        AppState::Error { .. } => Box::new(ErrorHandler),
        AppState::Exiting => Box::new(ExitingHandler),
    }
}

/// Handler for Configuring state
pub struct ConfiguringHandler;

impl InputHandler for ConfiguringHandler {
    fn handle_key(&self, key: KeyEvent, _state: &AppState) -> Vec<StateEvent> {
        let mut events = Vec::new();

        match (key.code, key.modifiers) {
            // Start analysis
            (KeyCode::Enter, KeyModifiers::NONE) => {
                events.push(StateEvent::StartAnalysis);
            }

            _ => {}
        }

        events
    }
}

/// Handler for Analyzing state
pub struct AnalyzingHandler;

impl InputHandler for AnalyzingHandler {
    fn handle_key(&self, _key: KeyEvent, _state: &AppState) -> Vec<StateEvent> {
        // No user input during analysis except Ctrl+Q (handled globally)
        vec![]
    }
}

/// Handler for ViewingResults state
pub struct ResultsHandler;

impl InputHandler for ResultsHandler {
    fn handle_key(&self, key: KeyEvent, state: &AppState) -> Vec<StateEvent> {
        let mut events = Vec::new();

        if let AppState::ViewingResults {
            selected_index,
            results,
            ..
        } = state
        {
            match key.code {
                // Navigation
                KeyCode::Up | KeyCode::Char('j') if *selected_index > 0 => {
                    events.push(StateEvent::SelectFile(selected_index - 1));
                }
                KeyCode::Down | KeyCode::Char('k')
                    if *selected_index < results.len().saturating_sub(1) =>
                {
                    events.push(StateEvent::SelectFile(selected_index + 1));
                }
                KeyCode::Home => {
                    events.push(StateEvent::SelectFile(0));
                }
                KeyCode::End => {
                    if !results.is_empty() {
                        events.push(StateEvent::SelectFile(results.len() - 1));
                    }
                }
                KeyCode::PageUp => {
                    let new_index = selected_index.saturating_sub(10);
                    events.push(StateEvent::SelectFile(new_index));
                }
                KeyCode::PageDown => {
                    let new_index = (*selected_index + 10).min(results.len().saturating_sub(1));
                    events.push(StateEvent::SelectFile(new_index));
                }
                // Open file detail
                KeyCode::Enter => {
                    events.push(StateEvent::OpenSelectedFile);
                }

                // Actions
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    events.push(StateEvent::Reanalyze);
                }
                KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    events.push(StateEvent::OpenFileLocation);
                }

                // Sort mode cycling
                KeyCode::Char('s') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Cycle through sort modes
                    // Implementation will cycle: ByScore -> ByName -> ByPath -> ByScore
                }

                // Go back
                KeyCode::Esc => {
                    events.push(StateEvent::GoBack);
                }

                _ => {}
            }
        }

        events
    }
}

/// Handler for ViewingFileDetail state
pub struct FileDetailHandler;

impl InputHandler for FileDetailHandler {
    fn handle_key(&self, key: KeyEvent, _state: &AppState) -> Vec<StateEvent> {
        let mut events = Vec::new();

        match key.code {
            // Scrolling
            KeyCode::Up | KeyCode::Char('j') => {
                events.push(StateEvent::ScrollUp);
            }
            KeyCode::Down | KeyCode::Char('k') => {
                events.push(StateEvent::ScrollDown);
            }
            KeyCode::PageUp => {
                // Scroll multiple lines
                for _ in 0..10 {
                    events.push(StateEvent::ScrollUp);
                }
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
                    events.push(StateEvent::ScrollDown);
                }
            }

            // Open file location
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                events.push(StateEvent::OpenFileLocation);
            }

            // Go back
            KeyCode::Esc => {
                events.push(StateEvent::GoBack);
            }

            _ => {}
        }

        events
    }
}

/// Handler for Error state
pub struct ErrorHandler;

impl InputHandler for ErrorHandler {
    fn handle_key(&self, key: KeyEvent, _state: &AppState) -> Vec<StateEvent> {
        let mut events = Vec::new();

        match key.code {
            // Return to previous state or configuring
            KeyCode::Esc | KeyCode::Enter => {
                events.push(StateEvent::GoBack);
            }
            // Allow 'q' to quit from error state
            KeyCode::Char('q') => {
                events.push(StateEvent::Quit);
            }
            _ => {}
        }

        events
    }
}

/// Handler for Exiting state
pub struct ExitingHandler;

impl InputHandler for ExitingHandler {
    fn handle_key(&self, _key: KeyEvent, _state: &AppState) -> Vec<StateEvent> {
        // No events processed in exiting state
        vec![]
    }
}
