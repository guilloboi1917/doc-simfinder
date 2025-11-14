use std::path::PathBuf;

use doc_simfinder::{config::Config, file_walker::walk_from_root};

fn main() {
    let config = Config {
        search_path: PathBuf::from(r"D:\Noah\04_CodingStuff\RUST\doc-simfinder\testdata"),
        max_search_depth: 2,
        num_threads: 1,
        file_exts: vec!["*.txt".to_string(), "*.md".to_string()],
        output_file: None,
        query: "test string".to_string(),
        algorithm: doc_simfinder::config::SimilarityAlgorithm::Levenshtein,
        threshold: Some(0.5_f64),
    };

    match walk_from_root(&config) {
        Ok(res) => println!("{}", res),
        Err(err) => println!("{}", err),
    };
}
