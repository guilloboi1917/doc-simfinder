> **Note:** This README was AI-generated to document the implemented solution.

# Files Module

## Module: `src/file_walker/mod.rs`

**Purpose**: Directory traversal + extension filtering → file list

## Types

```rust
pub struct WalkResult {
    pub files: Vec<PathBuf>,
    pub max_depth: usize,
}
```

## API
```rust
fn walk_from_root(config: &Config) -> Result<WalkResult, WalkError>
// Uses: walkdir + globset
// Config: search_path, file_exts, max_search_depth
// Returns: Vec<PathBuf> of matching files
```

**Algorithm**: Glob pattern from extensions → WalkDir → filter files → collect paths

## Integration

**State Machine**:
```rust
StateEvent::StartAnalysis → walk_from_root(&config) → files → analyse_files()
```

**Errors**: `WalkError::GlobPattern`, `WalkError::Io` (from thiserror)
