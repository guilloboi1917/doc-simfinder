#[derive(Debug, thiserror::Error)]
pub enum WalkError {
    #[error("Failed to build glob pattern: {0}")]
    GlobBuild(#[from] globset::Error),
    #[error("WalkDir error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ChunkError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ScoreError {
    #[error("Error processing chunks: {0}")]
    ChunkError(#[from] ChunkError),
}
