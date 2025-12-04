#[derive(Debug, thiserror::Error)]
pub enum WalkError {
    #[error("Failed to build glob pattern: {0}")]
    GlobBuild(#[from] globset::Error),
    #[error("File walk error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ChunkError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("File contains invalid UTF-8: {0}")]
    InvalidUtf8(String),
    #[error("File appears to be binary: {0}")]
    BinaryFile(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ScoreError {
    #[error("Error processing chunks: {0}")]
    ChunkError(#[from] ChunkError),
}
