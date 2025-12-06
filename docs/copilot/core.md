> **Note:** This README was AI-generated to document the implemented solution.

# Core Analysis Module

## Module: `src/analysis/mod.rs`

**Purpose**: Document similarity scoring via sliding-window chunking + fuzzy matching

## Key Types

```rust
pub struct FileScore {
    pub path: PathBuf,
    pub score: f64,              // Normalized [0.0, 1.0]
    pub top_chunks: Vec<ScoredChunk>,
    pub analysis_duration: Option<Duration>,
}

pub struct ScoredChunk {
    pub score: f64,
    pub chunk: Chunk,
    pub indices: Option<Vec<usize>>,  // For highlighting
}

pub struct Chunk {
    pub text: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

pub struct SlidingWindow {
    pub window_size: usize,
    pub overlap: usize,
}
```

## Public API

```rust
// Primary entrypoints
fn analyse_files(files: &Vec<PathBuf>, config: &Config) -> Result<Vec<FileScore>, ScoreError>
fn score_file(file: &Path, config: &Config) -> Result<FileScore, ScoreError>

// Algorithm: window → chunks → score → normalize → filter → top N
// Parallel: rayon with min 2 files/thread, 50 chunks/thread
// Error handling: Gracefully skips files with invalid UTF-8 or binary content
```

## Error Handling

**New as of 2025-12-04**: Robust handling of problematic files during analysis.

### Binary File Detection
- `is_likely_binary(file: &Path)` checks first 8KB of file
- Detects null bytes or >30% non-printable characters
- Prevents attempting UTF-8 read on binary files

### Error Types (from `src/errors/mod.rs`)
```rust
pub enum ChunkError {
    Io(std::io::Error),               // General IO errors
    InvalidUtf8(String),              // File path with invalid UTF-8
    BinaryFile(String),               // Binary file path
}
```

### Graceful Degradation
- `analyse_files()` uses `filter_map()` to skip problematic files
- Errors logged to stderr with "Warning: Skipping file -" prefix
- Processing continues with remaining valid files
- No panics on encountering invalid UTF-8 or binary files

## Integration

**Config fields used**: `query`, `algorithm`, `threshold`, `window_size`, `max_window_size`, `top_n`

**State Machine**: Called during `Analyzing` state transition
```rust
let files = walk_from_root(&config)?;
let results = analyse_files(&files, &config)?;
// → transition to ViewingResults { results, .. }
```

**TUI async pattern**:
```rust
tokio::task::spawn_blocking(move || analyse_files(&files, &config)).await?
```

**Presentation**: Use `format_file_result()` and `format_match_line()` from `presentation` module
