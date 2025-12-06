use crate::{config::Config, errors::WalkError};
use globset::{Glob, GlobSetBuilder};
use std::{fmt, path::PathBuf};
use jwalk::WalkDir;

#[derive(Debug, Clone)]
pub struct WalkResult {
    pub files: Vec<PathBuf>,
    pub max_depth: usize,
}

impl fmt::Display for WalkResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "WalkResult (max_depth: {})", self.max_depth)?;
        writeln!(f, "Files found: {}", self.files.len())?;

        if !self.files.is_empty() {
            writeln!(f, "Files:")?;
            for file in &self.files {
                if let Some(file_str) = file.to_str() {
                    writeln!(f, "  - {}", file_str)?;
                } else {
                    writeln!(f, "  - <invalid UTF-8>")?;
                }
            }
        } else {
            writeln!(f, "No files found.")?;
        }

        Ok(())
    }
}

// Recursively walk from root path
pub fn walk_from_root(config: &Config) -> Result<WalkResult, WalkError> {
    // new WalkResult
    let mut walk_result = WalkResult {
        files: Vec::new(),
        max_depth: 0,
    };

    let mut glob_builder = GlobSetBuilder::new();

    for ext in &config.file_exts {
        glob_builder.add(Glob::new(format!("*{}", ext).as_str())?); // from suffix (.txt) to glob pattern (*.txt)
    }

    let glob_set = glob_builder.build()?;

    // Use jwalk for parallel directory traversal (much faster for large trees)
    for entry in WalkDir::new(&config.search_path)
        .max_depth(config.max_search_depth)
        .into_iter()
        .filter_map(|e| e.ok()) // Skip errors silently
        .filter(|e| e.file_type().is_file() && glob_set.is_match(e.path()))
    {
        // Update max depth
        if entry.depth > walk_result.max_depth {
            walk_result.max_depth = entry.depth;
        }

        walk_result.files.push(entry.path());
    }

    Ok(walk_result)
}
