use std::path::PathBuf;

// The config struct is what's being created by either the interactive
// or the one-shot command.
// It is used by the modules further down the pipeline (analysis, output, ...)
pub struct Config {
    // Search path
    pub search_path: PathBuf,
    // Maximimum search depth
    pub max_search_depth: usize,
    // Thread number
    pub num_threads: usize,
    // File extensions
    pub file_exts: Vec<String>,
    // Output file
    pub output_file: Option<PathBuf>,

    // Query string
    pub query: String,

    // Analysis algorithm
    pub algorithm: SimilarityAlgorithm,
    // Matching threshold [0..1]
    pub threshold: Option<f64>,
}

impl Config {
    pub fn validate() -> Result<(), ConfigError> {
        todo!()
    }
}

pub enum SimilarityAlgorithm {
    Levenshtein,
    DamerauLevenshtein,
}

#[derive(Debug, Clone)]
pub struct ConfigError;
