# State Machine

## Module: `src/state_machine/`

**Pattern**: Event-driven state transitions, Config carried through all states

## States
```rust
pub enum AppState {
    Configuring { config, validation_errors },
    Analyzing { config, path, query, files_processed, total_files },
    ViewingResults { config, results, selected_index, sort_mode, filter },
    ViewingFileDetail { config, file_result, scroll_position },
    Error { message, previous_state },
    Exiting,
}
```

## Events
```rust
pub enum StateEvent {
    // Config: UpdatePath, UpdateQuery, ValidateConfig, StartAnalysis
    // Analysis (sent by background task): AnalysisProgress{files_done, total}, AnalysisComplete(Vec<FileScore>), AnalysisError(String)
    // Navigation: SelectFile(usize), OpenSelectedFile, GoBack
    // View: ChangeSortMode, SetFilter, ScrollUp, ScrollDown
    // Actions: SaveResults, ExportResults, Reanalyze
    // Global: ShowHelp, HideHelp, Quit
}
```

## Transition Table

```
Configuring --[StartAnalysis]--> Analyzing
Analyzing --[AnalysisComplete]--> ViewingResults
Analyzing --[AnalysisError]--> Error
ViewingResults --[OpenSelectedFile]--> ViewingFileDetail
ViewingResults --[Reanalyze]--> Analyzing
ViewingResults/FileDetail --[GoBack]--> Configuring
Any --[Quit]--> Exiting
```

## Implementation

```rust
pub struct StateMachine {
    current_state: AppState,
    event_queue: VecDeque<StateEvent>,
}

// transitions.rs: fn transition(&mut AppState, StateEvent) -> TransitionResult
// handlers.rs: trait InputHandler + get_handler_for_state()
```

## Background Analysis Execution

When `StartAnalysis` event is triggered (user presses Enter in Configuring state):

1. **TUI spawns background task** (`App::run_analysis_task`)
2. **Task performs file discovery** using `file_walker::walk_from_root()`
3. **Task sends progress update** via channel: `AnalysisProgress{files_done: 0, total: N}`
4. **Task runs analysis** in `spawn_blocking()`: `analysis::analyse_files()`
5. **Task sends completion** via channel: `AnalysisComplete(results)` or `AnalysisError(msg)`
6. **Main event loop receives** events from channel and processes through state machine

**Key Design Points**:
- Background task communicates exclusively via `StateEvent` channel (no shared state)
- CPU-bound analysis runs in `spawn_blocking()` to avoid blocking tokio runtime
- If no files found, sends `AnalysisError` with helpful message showing path and expected extensions
- State machine transitions are synchronous; async work happens outside state machine
