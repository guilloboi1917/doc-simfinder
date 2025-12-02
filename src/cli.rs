use std::path::PathBuf;

use crate::config::{Config, SimilarityAlgorithm};

#[derive(clap::Parser, Debug)]
#[command(name = "doc-simfinder")]
pub struct CliArgs {
    /// Path to search
    #[arg(long, default_value = ".")]
    pub search_path: PathBuf,

    /// Query string to search for
    #[arg(long)]
    pub query: Option<String>,

    /// Interactive mode
    #[arg(long, short, action)]
    pub interactive: bool,

    /// Window size in characters
    #[arg(long, default_value_t = 500)]
    pub window_size: usize,

    /// Maximum window size
    #[arg(long, default_value_t = 5000)]
    pub max_window_size: usize,

    /// File extensions to include (comma separated)
    #[arg(long, value_delimiter = ',')]
    pub file_exts: Vec<String>,

    /// Similarity algorithm
    #[arg(long, value_enum, default_value_t = Algorithm::Fuzzy)]
    pub algorithm: Algorithm,

    /// Threshold
    #[arg(long, short, default_value_t = 0.5_f64)]
    pub threshold: f64,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum Algorithm {
    Fuzzy,
    Lcs,
}

impl From<Algorithm> for SimilarityAlgorithm {
    fn from(a: Algorithm) -> SimilarityAlgorithm {
        match a {
            Algorithm::Fuzzy => SimilarityAlgorithm::Fuzzy,
            Algorithm::Lcs => SimilarityAlgorithm::LCS,
        }
    }
}

pub fn build_config_from_args(args: &CliArgs) -> Config {
    let file_exts = if args.file_exts.is_empty() {
        vec![".txt".to_string(), ".md".to_string()]
    } else {
        args.file_exts.clone()
    };

    Config {
        search_path: args.search_path.clone(),
        query: args.query.clone().unwrap_or("default".to_string()),
        window_size: args.window_size,
        max_window_size: args.max_window_size,
        file_exts,
        algorithm: args.algorithm.clone().into(),
        threshold: args.threshold,
        ..Default::default()
    }
}
