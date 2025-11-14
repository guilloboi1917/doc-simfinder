use std::{
    fs::{self},
    path::PathBuf,
};

use rayon::prelude::*;
use strsim::{normalized_damerau_levenshtein, normalized_levenshtein};

use crate::config::{Config, SimilarityAlgorithm};

// Return a score for each file
// Needs a weighting function for multiple matches within a file
pub fn analyse_files(files: &Vec<PathBuf>, config: &Config) {
    // might want to benchmark this
}

// Stream with BufReader
// Create set of chunks
// Run algo on chunks using rayon
fn analyse_file(query: String, file: PathBuf, algo: SimilarityAlgorithm) {
    match algo {
        SimilarityAlgorithm::Levenshtein => todo!(),
        SimilarityAlgorithm::DamerauLevenshtein => todo!(),
    }
}

fn get_chunks(file: PathBuf, window: &SlidingWindow) -> Result<Vec<Chunk>, std::io::Error> {
    // Use thiserror later
    // For convenience we load the whole file, maybe put some file size restrictions
    let content = fs::read_to_string(file).expect("failed to read to string");
    let chars: Vec<char> = content.chars().collect();

    let mut chunks: Vec<Chunk> = Vec::new();
    let mut start = 0;

    while start < chars.len() {
        let end = (start + window.window_size).min(chars.len());
        let chunk_text: String = chars[start..end].iter().collect();

        chunks.push(Chunk {
            text: chunk_text,
            start_byte: start,
            end_byte: end,
        });

        if end == chars.len() {
            break;
        }

        start = end.saturating_sub(window.overlap);
    }

    Ok(chunks)
}

fn score_chunk(chunk: Chunk) -> f64 {
    todo!()
}

fn score_file(file: PathBuf) -> FileScore {
    todo!()
}

// Use chunking to split a file into multiple chunks with overlap
// We can use a sliding window with overlap for this
// Makes it easier to extract context
pub struct SlidingWindow {
    pub window_size: usize, // in characters
    pub overlap: usize,     // in characters
}

// Think of tradeoffs, storing chunk data
// or only references using start_byte, end_byte and read from it later.
pub struct Chunk {
    pub text: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

pub struct ScoredChunk {
    pub score: f64,
    pub chunk: Chunk,
}

pub struct FileScore {
    pub path: PathBuf,
    pub score: f64,
    pub top_chunks: Vec<ScoredChunk>,
}
