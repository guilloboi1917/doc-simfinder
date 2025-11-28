use std::path::PathBuf;

use doc_simfinder::analysis::{Chunk, FileScore, ScoredChunk};
use doc_simfinder::config::Config;
use doc_simfinder::presentation::present_file_score;

#[test]
fn test_present_file_score_basic() {
    let chunk = Chunk {
        text: "This is a test snippet".to_string(),
        start_byte: 0,
        end_byte: 21,
    };

    let scored = ScoredChunk {
        score: 0.75,
        chunk,
        indices: None,
    };

    let fs = FileScore {
        path: PathBuf::from("test.txt"),
        score: 0.75,
        top_chunks: vec![scored],
    };

    let output = present_file_score(&fs, &Config::default());
    assert!(output.contains("File: test.txt"));
    assert!(output.contains("Top chunks"));
    assert!(output.contains("1."));
    assert!(output.contains("This is a test snippet"));
}
