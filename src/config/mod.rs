use std::path::PathBuf;

// The config struct is what's being created by either the interactive
// or the one-shot command.
// It is used by the modules further down the pipeline (analysis, output, ...)
#[derive(Debug, Clone)]
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
    pub threshold: f64,
    // Window size
    pub window_size: usize,
    // Maximum window size
    pub max_window_size: usize,

    // Number of top N chunks per file
    pub top_n: usize,
}

// Allowed file extensions
pub static ALLOWED_UTF8_FILE_EXTS: &[&str] = &[
    ".txt", ".md", ".rs", ".py", ".java", ".c", ".cpp", ".js", ".ts", ".html", ".css", ".json",
    ".yaml", ".yml", ".toml", ".xml",
];

// Currently we only allow for PDF as binary file extension
pub static ALLOWED_BINARY_FILE_EXTS: &[&str] = &[".pdf"];

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

        if self.file_exts.is_empty() {
            return Err(ConfigError);
        }

        if self.threshold < 0.0 || self.threshold > 1.0 {
            return Err(ConfigError);
        }

        if self.top_n == 0 {
            return Err(ConfigError);
        }

        if self.max_search_depth == 0 {
            return Err(ConfigError);
        }

        if self.file_exts.iter().any(|ext| {
            !ALLOWED_BINARY_FILE_EXTS.contains(&ext.as_str())
                && !ALLOWED_UTF8_FILE_EXTS.contains(&ext.as_str())
        }) {
            return Err(ConfigError);
        }

        // More validation needed here...

        Ok(())
    }
}

// Default values for config
impl Default for Config {
    fn default() -> Self {
        Self {
            search_path: Default::default(),
            max_search_depth: 5,
            num_threads: 0, // 0 means all threads are used
            file_exts: vec![".txt".to_string(), ".md".to_string()], // TODO! extend these
            output_file: None,
            query: Default::default(),
            algorithm: SimilarityAlgorithm::Fuzzy,
            threshold: 0.75_f64,
            window_size: 500,
            max_window_size: 5000,
            top_n: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SimilarityAlgorithm {
    Fuzzy,
    LCS,
}

// Put in errors.rs
#[derive(Debug, Clone)]
pub struct ConfigError;
