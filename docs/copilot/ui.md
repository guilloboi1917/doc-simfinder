# UI Module

## Presentation (`src/presentation/mod.rs`)

**Purpose**: Format analysis results for display (CLI + TUI)

```rust
fn get_terminal_width() -> usize
fn format_file_result(result: &FileScore) -> String
fn format_match_line(chunk: &Chunk, indices: &[usize]) -> String
```

Uses: `colored` (ANSI), `textwrap`, `term_size`

## Interactive (`src/interactive/mod.rs`)

**Current**: Simple `inquire`-based prompts (path, query, file selection)
**Future**: Keep as `--interactive`, add `--tui` for advanced mode

## TUI Widgets (`src/tui/widgets.rs`)

**Dashboard Pattern**: `Dashboard::new_for_state()` creates layout, `render()` delegates to state-specific methods

**Configuring State Widgets** (all 4 rendered):
1. `PathInput` - Editable text input, displays `config.search_path`, yellow border when focused
2. `QueryInput` - Editable text input, displays `config.query`, yellow border when focused
3. `OptionsPanel` - Read-only display of Config fields OR validation errors (red border)
4. `StartButton` - Validation indicator: ✓ green (ready) or ⚠ gray (invalid), shows requirements

**Results State Widgets**:
- `FileListWidget`: Scrollable file results with Up/Down/Enter (focusable)
- `FilePreviewWidget`: Chunk display with syntax highlighting, colored styling, automatic text wrapping (focusable)
- `StatsWidget`: Total/matched files, analysis duration (read-only, not focusable)
- `ActionPanel`: Available keyboard shortcuts including "Ctrl+O: Open Location" (read-only, not focusable)

**Focus Behavior**: Only `FileList` and `FilePreview` are focusable in ViewingResults state. Use Tab/Shift+Tab to cycle between them.

**Color Scheme** (matches CLI output from `presentation/mod.rs`):
- File paths: Cyan + Bold
- Analysis duration: Red + Italic
- Scores: Green + Bold (>0.0) or DarkGray
- Match indices: Yellow + Bold
- Match scores: Magenta
- Byte ranges: DarkGray
- Context headers: Underlined
- Chunk separators: DarkGray ("─")
- "No chunks" message: Yellow
- **Matched characters**: Yellow + Bold + Underline (character-level highlighting via `build_highlighted_spans()`)

**Text Wrapping**:
- Preview and detail views use `ratatui::widgets::Wrap { trim: false }` for automatic line wrapping
- Full text displayed (no artificial truncation) - wraps to pane width
- Highlighted matches remain visible regardless of text length
- Scrolling works correctly with wrapped content

**File Operations**:
- `Ctrl+O`: Open file's directory in system file manager (uses `opener` crate for cross-platform support)
- Works in both ViewingResults and ViewingFileDetail states

**Text Input Implementation**:
- **Config is the single source of truth** - no duplicate state
- Character capture in `App::handle_key()` checks `FocusManager::current()` first
- Updates Config directly: `config.query.push(c)` or `config.search_path = PathBuf::from(...)`
- **Rendering**: Widgets display from Config (`config.query`, `config.search_path`)
- Borrow checker pattern: check focus → get mutable config → modify
- Validation feedback appears in StartButton in real-time based on Config state

## Dashboard (`src/tui/layout.rs`)

```rust
pub struct Dashboard {
    panes: HashMap<PaneId, Box<dyn InteractiveWidget>>,
    layout: Layout,
}

fn new_for_state(state: &AppState) -> Self  // Builds widgets from state
fn render(&self, frame: &mut Frame, focus: &FocusManager)
```

## Event Loop (`src/tui/app.rs`)

```rust
pub async fn run_tui(mut app: App) -> Result<()>
```

Cycle: render → event::read() → handle_input → process_event → loop
