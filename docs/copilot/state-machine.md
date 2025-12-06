# State Machine (`src/state_machine/`)

**Pattern**: Event-driven transitions with Config threading through all states

## States
```rust
Configuring { config, validation_errors, walk_result, autocomplete... }
Analyzing { config, path, query, files_processed, total_files }
ViewingResults { config, results, selected_index, sort_mode, filter, total_duration }
ViewingFileDetail { config, file_result, scroll_position, previous_results }
Error { message, previous_state }
Exiting
```

## Key Events
**Config**: `UpdatePath`, `UpdateQuery`, `ValidateConfig`, `StartAnalysis`, `FileWalkComplete`  
**Analysis**: `AnalysisProgress`, `AnalysisComplete`, `AnalysisError`  
**Navigation**: `SelectFile`, `OpenSelectedFile`, `GoBack`  
**Actions**: `Reanalyze`, `OpenFileLocation`, `Quit`

## Transitions
```
Configuring --[StartAnalysis]--> Analyzing
Analyzing --[Complete/Error]--> ViewingResults/Error
ViewingResults --[OpenFile]--> ViewingFileDetail
ViewingResults --[Reanalyze]--> Analyzing
Any --[GoBack/Quit]--> Configuring/Exiting
```

## Background Execution
1. Spawn background task on `StartAnalysis`
2. Run file walk → send `AnalysisProgress`
3. Run analysis in `spawn_blocking()` → send `AnalysisComplete`
4. Main loop polls channel, processes events synchronously
