// State transition logic
//
// See docs/copilot/state-machine.md for transition patterns

use super::{AppState, StateEvent, SortMode};
use std::path::Path;

/// Open the file location in the system's default file manager
/// Uses the opener crate for cross-platform support (Windows, macOS, Linux)
fn open_file_location(path: &Path) {
    // Try to open the parent directory containing the file
    if let Some(parent) = path.parent() {
        let _ = opener::open(parent);
    }
}

/// Result of a state transition
#[derive(Debug)]
pub enum TransitionResult {
    /// State changed successfully
    Changed,
    /// No state change occurred (event not applicable)
    NoChange,
    /// Transition failed with error
    Error(String),
}

/// Main transition function that handles state changes based on events
pub fn transition(current_state: &mut AppState, event: StateEvent) -> TransitionResult {
    let new_state = match (&*current_state, event) {
        // Configuration -> Analyzing
        (AppState::Configuring { config, .. }, StateEvent::StartAnalysis) => {
            // Validate config before transitioning
            if let Err(_) = config.validate() {
                return TransitionResult::Error("Invalid configuration".into());
            }

            AppState::Analyzing {
                config: config.clone(),
                path: config.search_path.clone(),
                query: config.query.clone(),
                files_processed: 0,
                total_files: 0,
            }
        }

        // Analyzing -> ViewingResults
        (AppState::Analyzing { config, .. }, StateEvent::AnalysisComplete { mut results, elapsed }) => {
            // Filter out results below threshold
            results.retain(|r| r.score >= config.threshold);
            
            // Sort by score (descending - highest first)
            results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            
            AppState::ViewingResults {
                config: config.clone(),
                results,
                selected_index: 0,
                sort_mode: SortMode::ByScore,
                filter: None,
                total_duration: Some(elapsed),
            }
        }

        // Analyzing -> Error
        (AppState::Analyzing { .. }, StateEvent::AnalysisError(msg)) => {
            AppState::Error {
                message: msg,
                previous_state: Some(Box::new(current_state.clone())),
            }
        }

        // Progress updates within Analyzing state
        (
            AppState::Analyzing {
                config,
                path,
                query,
                ..
            },
            StateEvent::AnalysisProgress {
                files_done,
                total,
            },
        ) => {
            // Create new state with updated progress
            AppState::Analyzing {
                config: config.clone(),
                path: path.clone(),
                query: query.clone(),
                files_processed: files_done,
                total_files: total,
            }
        }

        // ViewingResults -> ViewingFileDetail
        (
            AppState::ViewingResults {
                config,
                results,
                selected_index,
                sort_mode,
                filter,
                total_duration,
            },
            StateEvent::OpenSelectedFile,
        ) => {
            if let Some(file_result) = results.get(*selected_index) {
                // Store the current ViewingResults state to return to it later
                let previous_results = Box::new(AppState::ViewingResults {
                    config: config.clone(),
                    results: results.clone(),
                    selected_index: *selected_index,
                    sort_mode: *sort_mode,
                    filter: filter.clone(),
                    total_duration: *total_duration,
                });
                
                AppState::ViewingFileDetail {
                    config: config.clone(),
                    file_result: file_result.clone(),
                    scroll_position: 0,
                    previous_results,
                }
            } else {
                return TransitionResult::Error("Invalid file selection".into());
            }
        }

        // Selection changes within ViewingResults
        (AppState::ViewingResults { config, results, sort_mode, filter, total_duration, .. }, StateEvent::SelectFile(index)) => {
            if index < results.len() {
                AppState::ViewingResults {
                    config: config.clone(),
                    results: results.clone(),
                    selected_index: index,
                    sort_mode: *sort_mode,
                    filter: filter.clone(),
                    total_duration: *total_duration,
                }
            } else {
                return TransitionResult::Error("Invalid file index".into());
            }
        }

        // Sort mode changes within ViewingResults
        (AppState::ViewingResults { config, results, selected_index, filter, total_duration, .. }, StateEvent::ChangeSortMode(new_mode)) => {
            AppState::ViewingResults {
                config: config.clone(),
                results: results.clone(),
                selected_index: *selected_index,
                sort_mode: new_mode,
                filter: filter.clone(),
                total_duration: *total_duration,
            }
        }

        // Filter changes within ViewingResults
        (AppState::ViewingResults { config, results, selected_index, sort_mode, total_duration, .. }, StateEvent::SetFilter(new_filter)) => {
            AppState::ViewingResults {
                config: config.clone(),
                results: results.clone(),
                selected_index: *selected_index,
                sort_mode: *sort_mode,
                filter: new_filter,
                total_duration: *total_duration,
            }
        }

        // Scrolling within ViewingFileDetail
        (AppState::ViewingFileDetail { config, file_result, scroll_position, previous_results, .. }, StateEvent::ScrollDown) => {
            AppState::ViewingFileDetail {
                config: config.clone(),
                file_result: file_result.clone(),
                scroll_position: scroll_position.saturating_add(1),
                previous_results: previous_results.clone(),
            }
        }

        (AppState::ViewingFileDetail { config, file_result, scroll_position, previous_results, .. }, StateEvent::ScrollUp) => {
            AppState::ViewingFileDetail {
                config: config.clone(),
                file_result: file_result.clone(),
                scroll_position: scroll_position.saturating_sub(1),
                previous_results: previous_results.clone(),
            }
        }

        // Go back transitions
        (AppState::ViewingFileDetail { previous_results, .. }, StateEvent::GoBack) => {
            // Return to the stored ViewingResults state
            *previous_results.clone()
        }

        (AppState::ViewingResults { config, .. }, StateEvent::GoBack) => {
            AppState::Configuring {
                config: config.clone(),
                validation_errors: vec![],
            }
        }

        // Reanalyze from results view
        (AppState::ViewingResults { config, .. }, StateEvent::Reanalyze) => {
            AppState::Analyzing {
                config: config.clone(),
                path: config.search_path.clone(),
                query: config.query.clone(),
                files_processed: 0,
                total_files: 0,
            }
        }

        // Open file location in Explorer (ViewingResults)
        (AppState::ViewingResults { results, selected_index, .. }, StateEvent::OpenFileLocation) => {
            if let Some(file_result) = results.get(*selected_index) {
                open_file_location(&file_result.path);
            }
            return TransitionResult::NoChange;
        }

        // Open file location in Explorer (ViewingFileDetail)
        (AppState::ViewingFileDetail { file_result, .. }, StateEvent::OpenFileLocation) => {
            open_file_location(&file_result.path);
            return TransitionResult::NoChange;
        }

        // Global quit event
        (_, StateEvent::Quit) => AppState::Exiting,

        // Unhandled event for current state
        _ => return TransitionResult::NoChange,
    };

    *current_state = new_state;
    TransitionResult::Changed
}
