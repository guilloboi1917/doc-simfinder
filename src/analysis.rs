use std::{
    fmt::Display,
    fs::{self},
    path::{Path, PathBuf},
};

use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use rayon::{iter::Chunks, prelude::*};

use crate::{
    config::{Config, SimilarityAlgorithm},
    errors::{ChunkError, ScoreError},
};

// Return a score for each file
// Needs a weighting function for multiple matches within a file
// Parallelize??
pub fn analyse_files(files: &Vec<PathBuf>, config: &Config) -> Result<Vec<FileScore>, ScoreError> {
    let mut results: Vec<FileScore> = Vec::new();
    for f in files {
        results.push(score_file(f, config)?);
    }
    Ok(results)
}

// Stream with BufReader
// Create set of chunks
// Run algo on chunks using rayon
pub fn score_file(file: &Path, config: &Config) -> Result<FileScore, ScoreError> {
    let query = &config.query;
    let sliding_window = calculate_sliding_window(query.len(), config);
    let chunks = get_chunks(file, &sliding_window)?; // Do better error handling here

    let query_str: &str = query; // Coerce once

    // Parallelize using rayon
    let mut scored_chunks: Vec<ScoredChunk> = chunks
        .par_iter()
        .with_min_len(50)
        .map(|chunk| {
            let score = score_chunk(query_str, &chunk, &config.algorithm);
            ScoredChunk {
                score: score.0,
                chunk: chunk.clone(),
            }
        })
        .collect();

    // Sort by score
    scored_chunks.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Ignore threshold for now
    // if let Some(mut threshold) = config.threshold {
    //     // threshold is percentage of top chunks
    //     scored_chunks.retain(|c| c.score >= threshold);
    // }

    scored_chunks.retain(|c| c.score > 0.0);

    // There might be no chunks above threshold
    if scored_chunks.is_empty() {
        return Ok(FileScore {
            path: file.to_path_buf(),
            score: 0.0,
            top_chunks: vec![],
        });
    }

    // Would be max score, can also use mean
    let file_score = scored_chunks[0].score;

    let top_chunks = scored_chunks.into_iter().take(config.top_n).collect();

    Ok(FileScore {
        path: file.to_path_buf(),
        score: file_score,
        top_chunks,
    })
}

// We want some dynamic window sizing based on the query string.
fn get_chunks(file: &Path, window: &SlidingWindow) -> Result<Vec<Chunk>, ChunkError> {
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

fn score_chunk(
    query: &str,
    chunk: &Chunk,
    algo: &SimilarityAlgorithm,
) -> (f64, Option<Vec<usize>>) {
    match algo {
        SimilarityAlgorithm::Fuzzy => {
            let matcher = SkimMatcherV2::default();
            match matcher.fuzzy_indices(&chunk.text, &query) {
                Some(res) => {
                    println!("{:?}", res);
                    (res.0 as f64, Some(res.1))
                }
                None => (0.0, None),
            }
        }
        // TODO
        SimilarityAlgorithm::LCS => (0.0, None),
    }
}

fn calculate_sliding_window(query_len: usize, config: &Config) -> SlidingWindow {
    let base = config.window_size;

    let min_size = query_len.saturating_add(query_len * 2);
    let max_size = config.max_window_size;

    let ws = base.max(min_size).min(max_size);

    // Overlap is 10% of window size (minimum 10 chars)
    SlidingWindow {
        window_size: ws,
        overlap: ws / 10,
    }
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
#[derive(Clone)]
pub struct Chunk {
    pub text: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Clone)]
pub struct ScoredChunk {
    pub score: f64,
    pub chunk: Chunk,
}

pub struct FileScore {
    pub path: PathBuf,
    pub score: f64,
    pub top_chunks: Vec<ScoredChunk>,
}

impl Display for FileScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "File \"{}\"", self.path.display())?;
        writeln!(f, "Score: {:.4}", self.score)?;

        if !self.top_chunks.is_empty() {
            writeln!(f, "Top chunks:")?;
            for chunk in &self.top_chunks {
                writeln!(f, "Chunk score: {}", chunk.score)?;
                writeln!(f, "Text: {}\n", chunk.chunk.text)?;
            }
        } else {
            writeln!(f, "No top chunks found.")?;
        }

        Ok(())
    }
}
