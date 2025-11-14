#[derive(Debug, thiserror::Error)]
pub enum WalkError {
    #[error("Failed to build glob pattern: {0}")]
    GlobBuild(#[from] globset::Error),
    #[error("WalkDir error: {0}")]
    WalkDir(#[from] walkdir::Error),
}
