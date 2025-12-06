# UI Components

## Presentation (`src/presentation/mod.rs`)
Formats results for CLI/TUI display using `colored`, `textwrap`, `term_size`

## TUI Widgets (`src/tui/widgets.rs`)

**Configuring State** (5 widgets):
- `PathInput`, `QueryInput` - Editable inputs, update Config directly
- `FileList` - Live preview of found files
- `OptionsPanel` - Config display or validation errors
- `StartButton` - Validation indicator (✓/⚠)

**Results State** (4 widgets):
- `FileListWidget` - Scrollable results (focusable, uses `ListState`)
- `FilePreviewWidget` - Chunk display with highlighting (focusable, wraps text)
- `StatsWidget` - File count, duration
- `ActionPanel` - Keyboard shortcuts

**Key Features**:
- Text wrapping via `textwrap::wrap()` before highlighting
- Config as single source of truth (no duplicate state)
- Autocomplete for paths (Tab to accept)
- `Ctrl+O` opens file location in system file manager

## Dashboard (`src/tui/layout.rs`)
Builds state-specific widget layouts, manages rendering and focus

## Event Loop (`src/tui/app.rs`)
Cycle: render → read event → handle input → process event → repeat
