// State machine module - manages application state transitions
//
// See docs/copilot/state-machine.md for patterns and integration guidelines

pub mod states;
pub mod transitions;
pub mod handlers;

use std::collections::VecDeque;

pub use states::*;
pub use transitions::*;
pub use handlers::*;

/// Main state machine struct that manages application state
pub struct StateMachine {
    current_state: AppState,
    event_queue: VecDeque<StateEvent>,
}

impl StateMachine {
    /// Create a new state machine with the given initial state
    pub fn new(initial_state: AppState) -> Self {
        Self {
            current_state: initial_state,
            event_queue: VecDeque::new(),
        }
    }

    /// Get a reference to the current state
    pub fn current_state(&self) -> &AppState {
        &self.current_state
    }

    /// Get a mutable reference to the current state
    pub fn current_state_mut(&mut self) -> &mut AppState {
        &mut self.current_state
    }

    /// Process a state event and potentially transition to a new state
    pub fn process_event(&mut self, event: StateEvent) -> TransitionResult {
        // Implementation will be in transitions.rs
        transition(&mut self.current_state, event)
    }

    /// Queue an event for later processing
    pub fn queue_event(&mut self, event: StateEvent) {
        self.event_queue.push_back(event);
    }

    /// Process all queued events
    pub fn process_queued_events(&mut self) -> Vec<TransitionResult> {
        let mut results = Vec::new();
        while let Some(event) = self.event_queue.pop_front() {
            results.push(self.process_event(event));
        }
        results
    }
}

#[cfg(test)]
mod tests;
