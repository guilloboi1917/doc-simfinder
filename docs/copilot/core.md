# Core Analysis (`src/analysis/mod.rs`)

**Purpose**: Document similarity scoring via sliding-window chunking + fuzzy matching

## API
```rust
fn analyse_files(files: &Vec<PathBuf>, config: &Config) -> Result<Vec<FileScore>, ScoreError>
fn score_file(file: &Path, config: &Config) -> Result<FileScore, ScoreError>
```

**Algorithm**: window → chunks → score → normalize → filter → top N  
**Parallel**: rayon (min 2 files/thread, 50 chunks/thread)  
**Error handling**: Skips invalid UTF-8/binary files gracefully

## Binary Detection
- Checks first 1KB for null bytes or >30% non-printable chars
- Extension pre-check for common binary types (.exe, .dll, etc.)
- Prevents UTF-8 read panics

## Integration
**Config**: `query`, `algorithm`, `threshold`, `window_size`, `max_window_size`, `top_n`  
**State Machine**: Called in `Analyzing` state  
**TUI**: Runs in `spawn_blocking()` to avoid blocking async runtime
