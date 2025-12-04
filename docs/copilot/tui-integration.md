# TUI Integration

## Architecture

```
App (event loop) → Dashboard (layout) → Widgets (rendering/input)
     ↓
Background Tasks (tokio) → StateEvent channel → StateMachine
```

**Integration**: Dashboard created from AppState, renders via FocusManager
**Background Execution**: File walking and analysis run in tokio tasks, communicate via mpsc channel

## Layout Patterns

**Results View**: 60% file list | 40% preview; stats + actions below
**Config View**: Centered input panel with validation feedback

## Input System

**Global**: Tab/BackTab (focus), Ctrl+Q (quit)
**Configuring**: Char/Backspace (edit PathInput/QueryInput), Enter (start analysis when valid)
**Results**: Ctrl+R (reanalyze), Ctrl+S (save), f (filter), s (sort)
**Detail**: j/k (scroll), PgUp/PgDn, g/G (top/bottom), n/p (next match)

### Text Input Pattern (Configuring State)
- Character input (`KeyCode::Char(c)`) handled in `App::handle_key()` before state handlers
- Checked against `FocusManager::current()` to determine target field
- **Updates Config directly** - no separate input buffers (Config is single source of truth)
- Widgets render directly from Config values (`config.search_path`, `config.query`)
- Config validation happens on every keystroke for real-time feedback in StartButton

## Widget Trait

```rust
pub trait InteractiveWidget {
    fn render(&self, area: Rect, is_focused: bool) -> Box<dyn Widget>;
    fn handle_input(&mut self, key: KeyEvent) -> Option<StateEvent>;
    fn set_focus(&mut self, focused: bool);
}
```

**Implementations**: FileListWidget, FilePreviewWidget, ConfigInputWidget, StatsWidget

## Dashboard Pattern

```rust
pub struct Dashboard {
    panes: HashMap<PaneId, Box<dyn InteractiveWidget>>,
    layout: LayoutConfig,
}

fn new_for_state(state: &AppState) -> Self  // Build widgets from state
fn render(&self, frame: &mut Frame, focus: &FocusManager)
fn handle_input(&mut self, key: KeyEvent, focus: &FocusManager) -> Vec<StateEvent>
```

## Terminal Setup

```rust
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>>  // Raw mode + alt screen
fn restore_terminal(terminal: &mut Terminal) -> Result<()>  // Cleanup
```

Uses: `crossterm` (terminal control), `ratatui` (rendering)

## Background Task Pattern

**Implementation in `App`**:
```rust
struct App {
    analysis_event_rx: mpsc::UnboundedReceiver<StateEvent>,
    analysis_event_tx: mpsc::UnboundedSender<StateEvent>,
    // ...
}

// Spawn on StartAnalysis event
tokio::spawn(async move {
    App::run_analysis_task(config, tx).await;
});

// Poll in main loop
while let Ok(event) = self.analysis_event_rx.try_recv() {
    self.state_machine.process_event(event);
}
```

**Task Execution Flow**:
1. `StartAnalysis` event → spawn background task with config clone
2. **State transitions to `Analyzing` with `files_processed=0, total_files=0`** (shows "Discovering files...")
3. Task calls `file_walker::walk_from_root()` (I/O bound)
4. Send `AnalysisProgress{files_done: 0, total: N}` → state updates with total_files count
5. Call `analysis::analyse_files()` in `spawn_blocking()` (CPU bound, may take several seconds)
6. Send `AnalysisComplete(results)` or `AnalysisError(msg)`
7. Main loop receives event, transitions state, updates UI

**Error Handling**: 
- No files found → `AnalysisError("No files found in <path> matching extensions [.txt, .md, ...]")` → transitions to Error state
- Analysis failure → `AnalysisError(error_msg)` → transitions to Error state
- Progress widget shows "Discovering files..." when `total_files == 0` (before file count known)
