use std::path::PathBuf;

use doc_simfinder::{analysis::score_file, config::Config, file_walker::walk_from_root};

#[test]
fn test_walk_from_root() {
    let mut cfg = Config::default();
    cfg.search_path = PathBuf::from("testdata");
    cfg.max_search_depth = 3;

    let res = walk_from_root(&cfg).expect("walk failed");
    assert!(res.files.len() >= 2, "expected at least 2 files in testdata");
}

#[test]
fn test_score_file_on_sample() {
    let mut cfg = Config::default();
    cfg.search_path = PathBuf::from("testdata");
    cfg.query = "Lorem ipsum".to_string();
    cfg.window_size = 200;
    cfg.num_threads = 1;

    let walk = walk_from_root(&cfg).expect("walk failed");
    // find file00.txt
    let file = walk
        .files
        .iter()
        .find(|p| p.file_name().map(|s| s.to_str().unwrap_or("") == "file00.txt").unwrap_or(false))
        .expect("file00.txt not found");

    let score = score_file(file.as_path(), &cfg).expect("scoring failed");
    // Expect a non-zero score for this query in file00.txt
    assert!(score.score > 0.0 || !score.top_chunks.is_empty(), "expected some matches");
}
