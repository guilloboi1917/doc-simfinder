<!-- .github/copilot-instructions.md for doc-simfinder -->
# doc-simfinder — Copilot instructions

# ⚠️ CRITICAL: Documentation Update Protocol (READ FIRST)

**EVERY code change, architectural decision, or bug fix MUST include documentation updates in the SAME response:**

## Required Updates Checklist
- [ ] **ALWAYS** add entry to `docs/copilot/worklog/2025-12.md` with:
  - Date and descriptive title
  - Changes made (bullet list)
  - Rationale for changes
  - List of affected files
  - Usage examples if applicable
  - Breaking changes if any
- [ ] Update relevant module docs in `docs/copilot/*.md` if:
  - APIs changed (function signatures, return types)
  - Architecture patterns changed
  - Integration points changed
- [ ] Update this file (`.github/copilot-instructions.md`) if:
  - Project structure changed
  - Major components added/removed
  - Developer workflows changed
  - Dependencies added/removed

**No exceptions. Documentation is not optional. Update docs BEFORE claiming task completion.**

---

Short: concurrent document similarity search (Rust). Help contributors by following the file-level patterns, public APIs, and conventions below.

**Big Picture**
- **Purpose**: Index and score documents by similarity to a query using sliding-window chunking + fuzzy matching (see `src/analysis/`).
- **Major components**: `src/file_walker/` (find files), `src/config/` (Config struct and defaults), `src/analysis/` (chunking and scoring, uses `rayon`), `src/cli/` (CLI arg parsing with `clap`), `src/presentation/` (formatted output with syntax highlighting), `src/state_machine/` (TUI state management), `src/tui/` (terminal UI with ratatui), `src/main.rs` (orchestrates CLI/TUI modes). Public modules are exported in `src/lib.rs`.

**Key implementation patterns**
- **Chunking & sliding window**: `analysis.rs` builds overlapping `Chunk`s from full file text (uses `fs::read_to_string`). Keep this pattern when changing scoring logic; functions like `get_chunks`, `calculate_sliding_window` and `score_chunk` are central.
- **Similarity normalization**: Scores are normalized against an approximate optimal score (`calculate_approximate_optimal_score`) before thresholding and selecting top N chunks (`top_n` from `Config`).
- **Parallel processing**: Use `rayon`'s `par_iter()` where present (scoring uses `par_iter()` with `.with_min_len(50)`). Preserve thread-safety and avoid global mutability.
- **Config-driven behavior**: `Config` (in `src/config.rs`) holds search parameters and defaults via `Default`. Many modules expect a `&Config` reference; prefer extending `Config` rather than scattering globals.

**Dependencies & integration points**
- See `Cargo.toml`: `clap` (CLI arg parsing), `colored` (ANSI coloring), `crossterm` (terminal control), `fuzzy-matcher` (similarity matching), `globset`, `inquire` (interactive prompts), `rayon` (parallelism), `tempfile`, `term_size`, `textwrap` (presentation formatting), `thiserror` (error handling), `walkdir` (file traversal).
- `analysis::score_file(path, &config)` is a primary entrypoint for computing similarity for a single file; `analysis::analyse_files(&[PathBuf], &config)` processes multiple files in parallel; `file_walker::walk_from_root(&config)` returns the input file set. Keep these function signatures stable when the CLI or other callers are added.

**Developer workflows (what works now)**
- Build: `cargo build` (from repo root).
- Run in CLI mode: `cargo run -- --query "your search text" [--search-path path] [other options]`.
- Run in TUI mode: `cargo run -- --tui` (interactive terminal UI with state machine).
- Tests: `cargo test` runs unit and integration tests in `tests/` directory (`config_tests.rs`, `integration_test.rs`, `presentation_unit.rs`).

**Project-specific guidance for edits**
- If you change chunking/IO, note that `analysis.rs` currently reads full files into memory. For large-file support prefer streaming/BufReader and update `get_chunks` accordingly.
- When adding CLI flags, implement in `src/cli.rs` using `clap` (already listed in `Cargo.toml`) and map flags into `Config`. `src/main.rs` contains a minimal manual example to mirror.
- Error types live in `src/errors.rs` and favor `thiserror` style; add variants there instead of ad-hoc strings.
- Keep public surface in `src/lib.rs` in sync with module exports when adding features.

**Examples (copyable patterns found in the repo)**
- Score a file from `main`: `let res = score_file(res.files[2].as_path(), &config);`
- Use defaults: `let cfg = Config::default();` — add or override fields as needed.

**What to watch for / current TODOs**
- `analysis/mod.rs` prints debug info with `println!` in several functions (`optimal score`, fuzzy match results); prefer `tracing` or conditional debug logging in PRs.
- TUI mode is implemented but needs progress indicators during analysis state.
- Terminal width detection in `presentation/mod.rs` falls back to 80 columns if detection fails.
- Old `interactive/mod.rs` module exists but is no longer used by main entry point (consider removal).

If anything above is unclear or you want more detail on a particular file, tell me which file or flow you want expanded and I will iterate the instructions.

## 🔄 State Machine & TUI Integration (New Feature)
### Architecture Overview
The application now has a state-driven TUI mode with multi-pane interface alongside the existing CLI mode. The old simple interactive mode has been replaced by TUI.
We're adding a state-driven interactive mode with multi-pane TUI alongside the existing CLI and simple interactive modes.

### Key Integration Points

#### 1. State Machine (`src/state_machine/`)
- **Purpose**: Manage application flow between configuration, analysis, and results viewing
- **Integration**: Uses existing `Config` struct and `analysis::analyse_files()`
- **Pattern**: State transitions triggered by user input or analysis completion

```rust
// Example state transition
AppState::Configuring -> StateEvent::StartAnalysis -> AppState::Analyzing
```

#### 2. TUI Layer (`src/tui/`)
- **Purpose**: Multi-pane terminal interface with focus management
- **Integration**: Renders current state, translates input to state events
- **Pattern**: Widget composition with Dashboard orchestrating layout
#### 3. Main Entry Point (`src/main.rs`)
- **Purpose**: Route between CLI and TUI modes
- **Integration**: `--tui` flag launches TUI, otherwise runs CLI mode
- **Pattern**: Async main with tokio runtime for TUI event loop
- **Pattern**: Tokio channels bridging file watcher to main loop

### File Structure
```
src/
├── main.rs             # Entry point, routes CLI vs TUI mode
├── state_machine/
│   ├── mod.rs          # Public API, StateMachine struct
│   ├── states.rs       # AppState enum definitions
│   ├── transitions.rs  # State transition logic
│   └── handlers.rs     # Input handlers per state
└── tui/
    ├── mod.rs          # Public API, setup/restore functions
    ├── app.rs          # App struct, main TUI event loop
    ├── layout.rs       # Dashboard and layout management
    ├── widgets.rs      # Interactive widget implementations
    └── focus.rs        # Focus management system
```

### Module-Specific Documentation
For detailed guidance on specific modules, refer to:
- `docs/copilot/core.md` - Analysis and scoring logic
- `docs/copilot/ui.md` - Terminal UI components and widgets
- `docs/copilot/files.md` - File system operations
- `docs/copilot/state-machine.md` - State machine patterns and transitions
- `docs/copilot/tui-integration.md` - TUI architecture and integration

### Development Workflow
- **Worklog**: See `docs/copilot/worklog/2025-12.md` for recent changes
- **Update Protocol**: See "Documentation Update Protocol" at the top of this file - MANDATORY for all changes

---

## Before You Finish ANY Task

Ask yourself: **"Did I update the worklog and relevant docs?"**

If the answer is NO, stop and update them NOW. Documentation updates are not optional follow-up work—they are part of completing the task.
