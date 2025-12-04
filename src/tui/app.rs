// Main TUI application struct
//
// See docs/copilot/tui-integration.md for application architecture

use std::io;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use crate::state_machine::{StateMachine, AppState, StateEvent};
use crate::{analysis, file_walker};
use super::{Dashboard, FocusManager, focus::FocusDirection};
use super::super::state_machine::handlers::get_handler_for_state;

/// Main TUI application
pub struct App {
    state_machine: StateMachine,
    pub(crate) focus_manager: FocusManager,
    pub(crate) should_quit: bool,
    needs_clear: bool,  // Track if terminal needs clearing on next render
    analysis_event_rx: mpsc::UnboundedReceiver<StateEvent>,
    analysis_event_tx: mpsc::UnboundedSender<StateEvent>,
}

impl App {
    /// Create a new TUI app with initial state
    pub fn new(initial_state: AppState) -> Self {
        let focus_manager = FocusManager::new_for_state(&initial_state);
        let state_machine = StateMachine::new(initial_state);
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            state_machine,
            focus_manager,
            should_quit: false,
            needs_clear: false,
            analysis_event_rx: rx,
            analysis_event_tx: tx,
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

            // Check for analysis events from background task
            while let Ok(event) = self.analysis_event_rx.try_recv() {
                let result = self.state_machine.process_event(event);
                if matches!(result, crate::state_machine::TransitionResult::Changed) {
                    self.needs_clear = true;
                    self.focus_manager = FocusManager::new_for_state(self.state_machine.current_state());
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
            KeyCode::Char('q') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return;
            }
            KeyCode::Tab => {
                self.focus_manager.move_focus(FocusDirection::Next);
                return;
            }
            KeyCode::BackTab => {
                self.focus_manager.move_focus(FocusDirection::Previous);
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
                KeyCode::Char(c) => {
                    use super::focus::Focus;
                    // Check focus first before borrowing state mutably
                    let current_focus = self.focus_manager.current();
                    match current_focus {
                        Focus::PathInput => {
                            if let AppState::Configuring { config, .. } = self.current_state_mut() {
                                let mut path_str = config.search_path.to_string_lossy().to_string();
                                path_str.push(c);
                                config.search_path = std::path::PathBuf::from(path_str);
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
                            if let AppState::Configuring { config, .. } = self.current_state_mut() {
                                let mut path_str = config.search_path.to_string_lossy().to_string();
                                path_str.pop();
                                config.search_path = std::path::PathBuf::from(path_str);
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
                if let AppState::Configuring { config, .. } = self.state_machine.current_state() {
                    let config_clone = config.clone();
                    let tx_clone = self.analysis_event_tx.clone();
                    tokio::spawn(async move {
                        Self::run_analysis_task(config_clone, tx_clone).await;
                    });
                }
            } else if matches!(event, StateEvent::Reanalyze) {
                // Get config from current state before transitioning
                if let Some(config) = self.state_machine.current_state().config() {
                    let config_clone = config.clone();
                    let tx_clone = self.analysis_event_tx.clone();
                    tokio::spawn(async move {
                        Self::run_analysis_task(config_clone, tx_clone).await;
                    });
                }
            }

            // Process the event through state machine
            let result = self.state_machine.process_event(event);

            // Update focus manager if state changed
            if matches!(result, crate::state_machine::TransitionResult::Changed) {
                self.needs_clear = true;
                self.focus_manager = FocusManager::new_for_state(self.state_machine.current_state());
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

    /// Background task that performs file walking and analysis
    async fn run_analysis_task(
        config: crate::config::Config,
        tx: mpsc::UnboundedSender<StateEvent>,
    ) {
        // Start tracking elapsed time
        let start_time = std::time::Instant::now();
        
        // First, walk the file system to find files
        let walk_result = match file_walker::walk_from_root(&config) {
            Ok(result) => result,
            Err(e) => {
                let _ = tx.send(StateEvent::AnalysisError(format!("File walk failed: {}", e)));
                return;
            }
        };

        let total_files = walk_result.files.len();

        // Send initial progress update with total file count
        let _ = tx.send(StateEvent::AnalysisProgress {
            files_done: 0,
            total: total_files,
        });

        // If no files found, send error
        if total_files == 0 {
            let _ = tx.send(StateEvent::AnalysisError(
                format!("No files found in {} matching extensions {:?}", 
                    config.search_path.display(), 
                    config.file_exts)
            ));
            return;
        }

        // Perform analysis using blocking task to avoid blocking tokio runtime
        let analysis_result = tokio::task::spawn_blocking(move || {
            analysis::analyse_files(&walk_result.files, &config)
        }).await;

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
