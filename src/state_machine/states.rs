// State definitions for the application state machine
//
// See docs/copilot/state-machine.md for state design patterns

use std::path::PathBuf;

use crate::config::Config;
use crate::analysis::FileScore;

/// Main application state enum
#[derive(Debug, Clone)]
pub enum AppState {
    /// Initial configuration state where user sets up search parameters
    Configuring {
        config: Config,
        validation_errors: Vec<String>,
    },

    /// Analysis in progress state
    Analyzing {
        config: Config,
        path: PathBuf,
        query: String,
        files_processed: usize,
        total_files: usize,
    },

    /// Viewing analysis results
    ViewingResults {
        config: Config,
        results: Vec<FileScore>,
        selected_index: usize,
        sort_mode: SortMode,
        filter: Option<String>,
        total_duration: Option<std::time::Duration>,
    },

    /// Viewing detailed information about a specific file
    ViewingFileDetail {
        config: Config,
        file_result: FileScore,
        scroll_position: usize,
        previous_results: Box<AppState>, // Store the ViewingResults state to return to
    },

    /// Error state with ability to recover to previous state
    Error {
        message: String,
        previous_state: Option<Box<AppState>>,
    },

    /// Terminal state - application is exiting
    Exiting,
}

impl AppState {
    /// Get a reference to the config, if available in this state
    pub fn config(&self) -> Option<&Config> {
        match self {
            AppState::Configuring { config, .. }
            | AppState::Analyzing { config, .. }
            | AppState::ViewingResults { config, .. }
            | AppState::ViewingFileDetail { config, .. } => Some(config),
            _ => None,
        }
    }

    /// Get a mutable reference to the config, if available
    pub fn config_mut(&mut self) -> Option<&mut Config> {
        match self {
            AppState::Configuring { config, .. } => Some(config),
            _ => None,
        }
    }
}

/// Sort mode for results view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    ByScore,
    ByName,
    ByPath,
}

/// Events that trigger state transitions
#[derive(Debug, Clone)]
pub enum StateEvent {
    // Configuration events
    UpdatePath(PathBuf),
    UpdateQuery(String),
    ValidateConfig,
    StartAnalysis,

    // Analysis events
    AnalysisProgress { files_done: usize, total: usize },
    AnalysisComplete { results: Vec<FileScore>, elapsed: std::time::Duration },
    AnalysisError(String),

    // Navigation events
    SelectFile(usize),
    OpenSelectedFile,
    GoBack,

    // View manipulation events
    ChangeSortMode(SortMode),
    SetFilter(Option<String>),
    ScrollUp,
    ScrollDown,

    // Action events
    SaveResults,
    ExportResults(PathBuf),
    Reanalyze,
    OpenFileLocation,

    // File system events (for real-time updates)
    FileChanged(PathBuf),
    FileCreated(PathBuf),
    FileDeleted(PathBuf),

    // Global events
    ShowHelp,
    HideHelp,
    Quit,
}
