# TUI Integration

## Architecture
```
App (event loop) → Dashboard → Widgets
      ↓
Background Tasks (tokio) → StateEvent channel → StateMachine
```

## Input Handling
**Global**: Ctrl+J/K (focus), Ctrl+Q (quit)  
**Configuring**: Char/Backspace (edit), Tab (autocomplete), Enter (start)  
**Results**: j/k (navigate), Ctrl+R (reanalyze), Ctrl+O (open location), Enter (detail)  
**Detail**: j/k (scroll), PgUp/PgDn, Ctrl+O (open location), Esc (back)

## Key Patterns

**Text Input**: Updates Config directly (single source of truth), no separate buffers

**Autocomplete**: Scans parent dir on keystroke, Tab accepts suggestion, triggers file walker

**Background Tasks**: File walk + analysis run in tokio tasks, communicate via `mpsc` channels

**Task Flow**:
1. `StartAnalysis` → spawn background task
2. File walk → `AnalysisProgress` event
3. Analysis (in `spawn_blocking()`) → `AnalysisComplete`/`AnalysisError`
4. Main loop polls channel, processes events

**Terminal Setup**: Uses `crossterm` (raw mode) + `ratatui` (rendering)
