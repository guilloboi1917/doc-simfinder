# State Machine

## Module: `src/state_machine/`

**Pattern**: Event-driven state transitions, Config carried through all states

## States
```rust
pub enum AppState {
    Configuring { 
        config, 
        validation_errors, 
        walk_result: Option<WalkResult>,
        autocomplete_available: bool,
        autocomplete_suggestion: Option<String>
    },
    Analyzing { config, path, query, files_processed, total_files },
    ViewingResults { config, results, selected_index, sort_mode, filter, total_duration },
    ViewingFileDetail { config, file_result, scroll_position, previous_results },
    Error { message, previous_state },
    Exiting,
}
```

### State Field Details

**Configuring State**:
- `walk_result`: Contains file list from background walker task, updated as user types path
- `autocomplete_available`: True when a single directory match exists for partial path
- `autocomplete_suggestion`: Full path suggestion to display (with normalized separators)

**ViewingResults State**:
- `total_duration`: Tracks analysis elapsed time for display in stats panel

**ViewingFileDetail State**:
- `previous_results`: Boxed ViewingResults state to return to on GoBack event

## Events
```rust
pub enum StateEvent {
    // Config: UpdatePath, UpdateQuery, ValidateConfig, StartAnalysis, FileWalkComplete{walk_result}
    // Analysis (sent by background task): AnalysisProgress{files_done, total}, AnalysisComplete{results, elapsed}, AnalysisError(String)
    // Navigation: SelectFile(usize), OpenSelectedFile, GoBack
    // View: ChangeSortMode, SetFilter, ScrollUp, ScrollDown
    // Actions: Reanalyze, OpenFileLocation
    // Global: ShowHelp, HideHelp, Quit
}
```

### Event Details

**FileWalkComplete**: Sent by background walker task when file discovery completes, contains `WalkResult` with list of found files

**AnalysisComplete**: Now includes `elapsed: Duration` field for performance tracking

**OpenFileLocation**: Opens file's parent directory in system file manager (Windows Explorer, macOS Finder, etc.)

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
