# File Walker (`src/file_walker/mod.rs`)

**Purpose**: Parallel directory traversal + extension filtering

## API
```rust
fn walk_from_root(config: &Config) -> Result<WalkResult, WalkError>
```

**Uses**: `jwalk` (parallel), `globset` (pattern matching)  
**Config**: `search_path`, `file_exts`, `max_search_depth`  
**Returns**: `WalkResult { files: Vec<PathBuf>, max_depth: usize }`

## Integration
**State Machine**: Triggered by `StartAnalysis` and path input changes  
**TUI**: Runs in background task, sends `FileWalkComplete` event  
**Errors**: `WalkError::GlobPattern`, `WalkError::Io`
