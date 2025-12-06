// Main TUI application struct
//
// See docs/copilot/tui-integration.md for application architecture

use crossterm::event::{self, Event, KeyCode};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tokio::sync::mpsc;

use super::super::state_machine::handlers::get_handler_for_state;
use super::{Dashboard, FocusManager, focus::FocusDirection};
use crate::state_machine::{AppState, StateEvent, StateMachine};
use crate::{analysis, file_walker};

/// Main TUI application
pub struct App {
    state_machine: StateMachine,
    pub(crate) focus_manager: FocusManager,
    pub(crate) should_quit: bool,
    needs_clear: bool, // Track if terminal needs clearing on next render
    analysis_event_rx: mpsc::UnboundedReceiver<StateEvent>,
    analysis_event_tx: mpsc::UnboundedSender<StateEvent>,
    walker_event_rx: mpsc::UnboundedReceiver<StateEvent>,
    walker_event_tx: mpsc::UnboundedSender<StateEvent>,
}

impl App {
    /// Create a new TUI app with initial state
    pub fn new(initial_state: AppState) -> Self {
        let focus_manager = FocusManager::new_for_state(&initial_state);
        let state_machine = StateMachine::new(initial_state);
        // Channel for receiving analysis events from background task
        let (tx_analysis, rx_analysis) = mpsc::unbounded_channel();
        // Channel for receiving walker events from background task
        let (tx_walker, rx_walker) = mpsc::unbounded_channel();

        Self {
            state_machine,
            focus_manager,
            should_quit: false,
            needs_clear: false,
            analysis_event_rx: rx_analysis,
            analysis_event_tx: tx_analysis,
            walker_event_rx: rx_walker,
            walker_event_tx: tx_walker,
        }
    }

    /// Run the TUI application
    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        loop {
            // Check if we should quit
            if self.should_quit || matches!(self.state_machine.current_state(), AppState::Exiting) {
                break;
            }

            // Clear terminal if state changed to prevent artifacts
            if self.needs_clear {
                terminal.clear()?;
                self.needs_clear = false;
            }

            // Render current state
            terminal.draw(|frame| {
                let dashboard = Dashboard::new_for_state(self.state_machine.current_state());
                dashboard.render(
                    frame,
                    self.state_machine.current_state(),
                    &self.focus_manager,
                );
            })?;

            // Check for walker events from background task
            while let Ok(event) = self.walker_event_rx.try_recv() {
                let result = self.state_machine.process_event(event);
                if matches!(result, crate::state_machine::TransitionResult::Changed) {
                    self.needs_clear = true;
                    self.focus_manager =
                        FocusManager::new_for_state(self.state_machine.current_state());
                }
            }

            // Check for analysis events from background task
            while let Ok(event) = self.analysis_event_rx.try_recv() {
                let result = self.state_machine.process_event(event);
                if matches!(result, crate::state_machine::TransitionResult::Changed) {
                    self.needs_clear = true;
                    self.focus_manager =
                        FocusManager::new_for_state(self.state_machine.current_state());
                }
            }

            // Handle input
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }
        }

        Ok(())
    }

    /// Handle keyboard input
    pub(crate) fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        // Only process key press events, ignore key release events
        if key.kind != crossterm::event::KeyEventKind::Press {
            return;
        }

        // Global shortcuts that work in any state
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('j') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.focus_manager.move_focus(FocusDirection::Previous);
                return;
            }
            KeyCode::Char('k') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.focus_manager.move_focus(FocusDirection::Next);
                return;
            }
            _ => {}
        }

        // State-specific input handling (handles 'q' for quit, Enter for start, etc.)
        let handler = get_handler_for_state(self.state_machine.current_state());
        let events = handler.handle_key(key, self.state_machine.current_state());

        // If no events generated, try text input handling for editable fields
        if events.is_empty() {
            match key.code {
                KeyCode::Tab => {
                    use super::focus::Focus;
                    // Handle Tab for autocomplete when PathInput is focused
                    if self.focus_manager.is_focused(Focus::PathInput) {
                        if let AppState::Configuring {
                            config,
                            autocomplete_available,
                            autocomplete_suggestion,
                            ..
                        } = self.current_state_mut()
                        {
                            if *autocomplete_available {
                                if let Some(suggestion) = autocomplete_suggestion.clone() {
                                    config.search_path = std::path::PathBuf::from(&suggestion);
                                    *autocomplete_available = false;
                                    *autocomplete_suggestion = None;

                                    // Trigger file walker for new path
                                    let config_clone = config.clone();
                                    let tx_clone = self.walker_event_tx.clone();
                                    tokio::spawn(async move {
                                        Self::run_filewalker_task(config_clone, tx_clone).await;
                                    });
                                }
                            }
                        }
                        return;
                    }
                }
                KeyCode::Char(c) => {
                    use super::focus::Focus;
                    // Check focus first before borrowing state mutably
                    let current_focus = self.focus_manager.current();
                    match current_focus {
                        Focus::PathInput => {
                            if let AppState::Configuring {
                                config,
                                autocomplete_available,
                                autocomplete_suggestion,
                                ..
                            } = self.current_state_mut()
                            {
                                let mut path_str = config.search_path.to_string_lossy().to_string();
                                path_str.push(c);
                                config.search_path = std::path::PathBuf::from(&path_str);

                                // Update autocomplete suggestions
                                Self::update_autocomplete(
                                    &path_str,
                                    &config.search_path,
                                    autocomplete_available,
                                    autocomplete_suggestion,
                                );

                                // Trigger file walker for new path
                                let config_clone = config.clone();
                                let tx_clone = self.walker_event_tx.clone();
                                tokio::spawn(async move {
                                    Self::run_filewalker_task(config_clone, tx_clone).await;
                                });
                                return;
                            }
                        }
                        Focus::QueryInput => {
                            if let AppState::Configuring { config, .. } = self.current_state_mut() {
                                config.query.push(c);
                                return;
                            }
                        }
                        _ => {} // Not in an input field
                    }
                }
                KeyCode::Backspace => {
                    use super::focus::Focus;
                    // Check focus first before borrowing state mutably
                    let current_focus = self.focus_manager.current();
                    match current_focus {
                        Focus::PathInput => {
                            if let AppState::Configuring {
                                config,
                                autocomplete_available,
                                autocomplete_suggestion,
                                ..
                            } = self.current_state_mut()
                            {
                                let mut path_str = config.search_path.to_string_lossy().to_string();
                                path_str.pop();
                                config.search_path = std::path::PathBuf::from(&path_str);

                                // Update autocomplete suggestions
                                Self::update_autocomplete(
                                    &path_str,
                                    &config.search_path,
                                    autocomplete_available,
                                    autocomplete_suggestion,
                                );

                                // Trigger file walker for modified path
                                let config_clone = config.clone();
                                let tx_clone = self.walker_event_tx.clone();
                                tokio::spawn(async move {
                                    Self::run_filewalker_task(config_clone, tx_clone).await;
                                });
                                return;
                            }
                        }
                        Focus::QueryInput => {
                            if let AppState::Configuring { config, .. } = self.current_state_mut() {
                                config.query.pop();
                                return;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Process all generated events
        for event in events {
            // Handle quit event specially
            if matches!(event, StateEvent::Quit) {
                self.should_quit = true;
                break;
            }

            // If StartAnalysis or Reanalyze event, spawn background task
            if matches!(event, StateEvent::StartAnalysis) {
                if let AppState::Configuring {
                    config,
                    walk_result,
                    ..
                } = self.state_machine.current_state()
                {
                    // Only start analysis if we have walk results
                    if let Some(walk_result) = walk_result {
                        let config_clone = config.clone();
                        let walk_result_clone = walk_result.clone();
                        let tx_clone = self.analysis_event_tx.clone();
                        tokio::spawn(async move {
                            Self::run_analysis_task(config_clone, walk_result_clone, tx_clone)
                                .await;
                        });
                    }
                }
            } else if matches!(event, StateEvent::Reanalyze) {
                // For reanalyze, we need to trigger a new file walk first
                if let Some(config) = self.state_machine.current_state().config() {
                    let config_clone = config.clone();
                    let tx_walker = self.walker_event_tx.clone();

                    tokio::spawn(async move {
                        // First run file walker
                        Self::run_filewalker_task(config_clone.clone(), tx_walker).await;
                        // Analysis will be triggered after FileWalkComplete is processed
                    });
                }
            }

            // Process the event through state machine
            let result = self.state_machine.process_event(event);

            // Update focus manager if state changed
            if matches!(result, crate::state_machine::TransitionResult::Changed) {
                self.needs_clear = true;
                self.focus_manager =
                    FocusManager::new_for_state(self.state_machine.current_state());
            }
        }
    }

    /// Get a reference to the current state
    pub fn current_state(&self) -> &AppState {
        self.state_machine.current_state()
    }

    /// Get a mutable reference to the current state (private helper)
    fn current_state_mut(&mut self) -> &mut AppState {
        self.state_machine.current_state_mut()
    }

    /// Update autocomplete suggestions based on current path
    fn update_autocomplete(
        path_str: &str,
        current_path: &std::path::Path,
        autocomplete_available: &mut bool,
        autocomplete_suggestion: &mut Option<String>,
    ) {
        *autocomplete_available = false;
        *autocomplete_suggestion = None;

        if current_path.is_dir() {
            return;
        }

        let Some(parent) = current_path.parent() else {
            return;
        };

        let Some(partial_name) = current_path.file_name().and_then(|n| n.to_str()) else {
            return;
        };

        if partial_name.is_empty() {
            return;
        }

        let Ok(read_dir) = std::fs::read_dir(parent) else {
            return;
        };

        let matches: Vec<std::path::PathBuf> = read_dir
            .flatten()
            .filter(|entry| {
                entry.path().is_dir()
                    && entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(partial_name)
            })
            .map(|entry| entry.path())
            .collect();

        if matches.len() == 1 {
            *autocomplete_available = true;
            // Normalize path separators to match user input style
            let suggestion = matches.first().unwrap().to_string_lossy().to_string();
            let normalized = if path_str.contains('/') {
                suggestion.replace('\\', "/")
            } else {
                suggestion
            };
            *autocomplete_suggestion = Some(normalized);
        }
    }

    async fn run_filewalker_task(
        config: crate::config::Config,
        tx: mpsc::UnboundedSender<StateEvent>,
    ) {
        // Perform file walking in a blocking task to avoid blocking tokio runtime
        let walk_result =
            tokio::task::spawn_blocking(move || file_walker::walk_from_root(&config)).await;

        match walk_result {
            Ok(Ok(result)) => {
                // Send event with walk result
                let _ = tx.send(StateEvent::FileWalkComplete {
                    walk_result: result,
                });
            }
            Ok(Err(e)) => {
                let _ = tx.send(StateEvent::AnalysisError(format!(
                    "File walk failed: {}",
                    e
                )));
            }
            Err(e) => {
                let _ = tx.send(StateEvent::AnalysisError(format!("Task failed: {}", e)));
            }
        }
    }

    /// Background task that performs file walking and analysis
    async fn run_analysis_task(
        config: crate::config::Config,
        walk_result: file_walker::WalkResult,
        tx: mpsc::UnboundedSender<StateEvent>,
    ) {
        // Start tracking elapsed time
        let start_time = std::time::Instant::now();

        // Send initial progress update with total file count
        let _ = tx.send(StateEvent::AnalysisProgress {
            files_done: 0,
            total: walk_result.files.len(),
        });

        // If no files found, send error
        if walk_result.files.is_empty() {
            let _ = tx.send(StateEvent::AnalysisError(format!(
                "No files found in {} matching extensions {:?}",
                config.search_path.display(),
                config.file_exts
            )));
            return;
        }

        // Perform analysis using blocking task to avoid blocking tokio runtime
        let analysis_result = tokio::task::spawn_blocking(move || {
            analysis::analyse_files(&walk_result.files, &config)
        })
        .await;

        // Calculate elapsed time
        let elapsed = start_time.elapsed();

        match analysis_result {
            Ok(Ok(results)) => {
                // Send completion event with elapsed time
                let _ = tx.send(StateEvent::AnalysisComplete { results, elapsed });
            }
            Ok(Err(e)) => {
                let _ = tx.send(StateEvent::AnalysisError(format!("Analysis failed: {}", e)));
            }
            Err(e) => {
                let _ = tx.send(StateEvent::AnalysisError(format!("Task failed: {}", e)));
            }
        }
    }
}
