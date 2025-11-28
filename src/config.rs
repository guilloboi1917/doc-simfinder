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
    pub threshold: Option<f64>, // Probably will not be used when using fuzzymatcher
    // Window size
    pub window_size: usize,
    // Maximum window size
    pub max_window_size: usize,

    // Number of top N chunks per file
    pub top_n: usize,
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Basic validation used by CLI and programmatic callers
        if self.query.is_empty() {
            return Err(ConfigError);
        }

        if !self.search_path.exists() {
            return Err(ConfigError);
        }

        if self.window_size == 0 || self.max_window_size == 0 {
            return Err(ConfigError);
        }

        Ok(())
    }
}

// Default values for config
impl Default for Config {
    fn default() -> Self {
        Self {
            search_path: Default::default(),
            max_search_depth: 5,
            num_threads: 0,                                         // 0 means all
            file_exts: vec![".txt".to_string(), ".md".to_string()], // extend these
            output_file: None,
            query: Default::default(),
            algorithm: SimilarityAlgorithm::Fuzzy,
            threshold: Some(0.5_f64),
            window_size: 500,
            max_window_size: 5000,
            top_n: 5,
        }
    }
}

pub enum SimilarityAlgorithm {
    Fuzzy,
    LCS,
}

// Put in errors.rs
#[derive(Debug, Clone)]
pub struct ConfigError;
