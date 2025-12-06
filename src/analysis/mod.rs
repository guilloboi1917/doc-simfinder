use std::{
    fmt::Display,
    fs::{self, File},
    io::Read,
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    time::Instant,
};

use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use rayon::prelude::*;

use crate::{
    config::{ALLOWED_BINARY_FILE_EXTS, Config, SimilarityAlgorithm},
    errors::{ChunkError, ScoreError},
};

// Return a score for each file
// Needs a weighting function for multiple matches within a file
pub fn analyse_files(files: &Vec<PathBuf>, config: &Config) -> Result<Vec<FileScore>, ScoreError> {
    let results: Vec<Result<FileScore, ScoreError>> = files
        .par_iter()
        .with_min_len(2)
        .map(|f| {
            // Wrap each file processing in catch_unwind to handle panics
            // For some reason pdf_extract can panic on corrupted PDFs
            match std::panic::catch_unwind(AssertUnwindSafe(|| score_file(f, config))) {
                Ok(result) => result,
                Err(_) => Err(ScoreError::ChunkError(ChunkError::PdfProcessing(format!(
                    "Processing panicked for file: {}",
                    f.display()
                )))),
            }
        })
        .collect();

    // Filter out errors but log them
    let successful_results: Vec<FileScore> = results
        .into_iter()
        .filter_map(|result| {
            match result {
                Ok(score) => Some(score),
                Err(e) => {
                    // Log the error but continue processing other files
                    eprintln!("Warning: Skipping file - {}", e);
                    None
                }
            }
        })
        .collect();

    Ok(successful_results)
}

// Stream with BufReader
// Create set of chunks
// Run algo on chunks using rayon
pub fn score_file(file: &Path, config: &Config) -> Result<FileScore, ScoreError> {
    let start_time = Instant::now();
    let query = &config.query;
    let sliding_window = calculate_sliding_window(query.len(), config);

    let optimal_score =
        calculate_approximate_optimal_score(query.len(), sliding_window.window_size);
    let chunks = get_chunks(file, &sliding_window)?; // Do better error handling here

    let query_str: &str = query; // Coerce once

    // Parallelize using rayon
    let mut scored_chunks: Vec<ScoredChunk> = chunks
        .par_iter()
        .with_min_len(50)
        .map(|chunk| {
            // Normalize based on optimal score
            let (raw_score, indices_opt) = score_chunk(query_str, &chunk, &config.algorithm);
            let chunk_with_indices = chunk.clone();
            ScoredChunk {
                score: (raw_score / (optimal_score as f64)).clamp(0.0, 1.0),
                chunk: chunk_with_indices,
                indices: indices_opt,
            }
        })
        .collect();

    // Sort by score
    scored_chunks.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // threshold is percentage of top chunks
    scored_chunks.retain(|c| c.score >= config.threshold);

    // There might be no chunks above threshold
    if scored_chunks.is_empty() {
        return Ok(FileScore {
            path: file.to_path_buf(),
            score: 0.0,
            top_chunks: vec![],
            analysis_duration: None,
        });
    }

    // Would be max score, can also use mean
    let file_score = scored_chunks[0].score;

    let top_chunks = scored_chunks.into_iter().take(config.top_n).collect();

    Ok(FileScore {
        path: file.to_path_buf(),
        score: file_score,
        top_chunks,
        analysis_duration: Some(start_time.elapsed()),
    })
}

/// Check if a file appears to be binary by reading the first few bytes
/// Just in case we try to read a binary file as UTF-8 text
fn is_likely_binary(file: &Path) -> Result<bool, std::io::Error> {
    // Quick extension check first (avoids I/O for obvious cases)
    if let Some(ext) = file.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        // Common binary extensions
        if matches!(
            ext_lower.as_str(),
            "exe"
                | "dll"
                | "so"
                | "dylib"
                | "bin"
                | "obj"
                | "o"
                | "zip"
                | "tar"
                | "gz"
                | "7z"
                | "rar"
                | "bz2"
                | "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "bmp"
                | "ico"
                | "webp"
                | "mp3"
                | "mp4"
                | "avi"
                | "mkv"
                | "mov"
                | "flac"
                | "wav"
                | "pdf"
                | "doc"
                | "docx"
                | "xls"
                | "xlsx"
                | "ppt"
                | "pptx"
        ) {
            return Ok(true);
        }
    }

    let mut file = File::open(file)?;
    let mut buffer = [0u8; 1024]; // Check first 1KB
    let bytes_read = file.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(false); // Empty file, treat as text
    }

    // Check for null bytes (common in binary files)
    let has_null = buffer[..bytes_read].contains(&0);

    // Check for high ratio of non-printable characters
    let non_printable_count = buffer[..bytes_read]
        .iter()
        .filter(|&&b| b < 32 && b != b'\n' && b != b'\r' && b != b'\t')
        .count();

    let non_printable_ratio = non_printable_count as f64 / bytes_read as f64;

    Ok(has_null || non_printable_ratio > 0.3)
}

// We want some dynamic window sizing based on the query string.
fn get_chunks(file: &Path, window: &SlidingWindow) -> Result<Vec<Chunk>, ChunkError> {
    let file_ext = file.extension().unwrap_or_default().to_string_lossy();
    // Check if file is allowed and if not if it is likely binary before attempting to read as UTF-8
    if !ALLOWED_BINARY_FILE_EXTS.contains(&format!(".{}", &file_ext).as_str())
        && is_likely_binary(&file)?
    {
        return Err(ChunkError::BinaryFile(file.display().to_string()));
    }

    // TODO! I should refactor this
    // Quick implementation for project finishing
    let content = match file_ext.as_ref() {
        "pdf" => extract_pdf_text(file)?,
        _ => {
            // Attempt to read file as UTF-8 text
            read_text_file(file)?
        }
    };

    // More efficient: work with char indices directly instead of collecting all chars
    let char_indices: Vec<(usize, char)> = content.char_indices().collect();
    let char_count = char_indices.len();

    let mut chunks: Vec<Chunk> = Vec::new();
    let mut start_idx = 0;

    while start_idx < char_count {
        let end_idx = (start_idx + window.window_size).min(char_count);

        // Get byte positions for slicing
        let start_byte = char_indices[start_idx].0;
        let end_byte = if end_idx < char_count {
            char_indices[end_idx].0
        } else {
            content.len()
        };

        let chunk_text = content[start_byte..end_byte].to_string();

        chunks.push(Chunk {
            text: chunk_text,
            start_byte: start_idx,
            end_byte: end_idx,
        });

        if end_idx == char_count {
            break;
        }

        start_idx = end_idx.saturating_sub(window.overlap);
    }

    Ok(chunks)
}

/// Calculate a spread penalty based on how dispersed the match indices are.
///
/// If matches are tightly clustered (spread <= query_len), penalty is 1.0 (no penalty).
/// If matches are spread far apart (spread > query_len), penalty decreases (stronger penalty).
///
/// Formula:
///   - If normalized_spread <= 1.0: penalty = 1.0 (no penalty)
///   - If normalized_spread > 1.0: penalty = 1.0 / normalized_spread
///
/// where normalized_spread is the ratio of (max - min + 1) to query length.
/// The +1 accounts for the actual range: indices 0 and 1 represent 2 characters.
fn calculate_spread_penalty(indices: &[usize], query_len: usize) -> f64 {
    if indices.len() <= 1 {
        return 1.0; // No spread penalty for single match or empty
    }

    let min_idx = *indices.iter().min().unwrap();
    let max_idx = *indices.iter().max().unwrap();
    // Needs +1 otherwise normalized spread can never be 1
    let spread = (max_idx - min_idx + 1) as f64;

    // Normalize spread relative to query length
    let normalized_spread = spread / (query_len as f64);

    // Penalty function:
    // - normalized_spread <= 1.0 => penalty = 1.0 (tight clustering, no penalty)
    // - normalized_spread > 1.0 => penalty = 1.0 / normalized_spread (penalize dispersion)
    // Examples:
    //   - indices at 0,1 (spread = 2, normalized = 2/query_len)
    //   - if query_len = 2: normalized = 1.0 => penalty = 1.0
    //   - if query_len = 2, spread = 4: normalized = 2.0 => penalty = 0.5
    if normalized_spread <= 1.0 {
        1.0
    } else {
        1.0 / normalized_spread
    }
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
                    // Apply spread penalty: penalize if matched chars are far apart
                    let base_score = res.0 as f64;
                    let spread_penalty = calculate_spread_penalty(&res.1, query.len());
                    let penalized_score = base_score * spread_penalty;
                    (penalized_score, Some(res.1))
                }
                None => (0.0, None),
            }
        }
        // TODO
        SimilarityAlgorithm::LCS => (0.0, None),
    }
}

fn calculate_approximate_optimal_score(query_len: usize, window_size: usize) -> i64 {
    let matcher = SkimMatcherV2::default();

    // input validation for querylen and windowsize?

    let binding = "2".repeat(query_len);
    let pattern = binding.as_str();

    // Create a string which holds an exact match
    let mut s_contain_match = "1".repeat(window_size - query_len);
    s_contain_match.push_str(pattern);

    match matcher.fuzzy_match(s_contain_match.as_str(), pattern) {
        Some(score) => {
            // println!("optimal score: {}", score);
            score
        }
        None => 0,
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

/// Extract text from a PDF file with panic recovery using lopdf
fn extract_pdf_text(file: &Path) -> Result<String, ChunkError> {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    // Check file size before processing
    let metadata = fs::metadata(file).map_err(|e| ChunkError::Io(e))?;
    
    const MAX_PDF_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
    if metadata.len() > MAX_PDF_SIZE {
        return Err(ChunkError::PdfProcessing(format!(
            "PDF too large ({}MB > 10MB)",
            metadata.len() / (1024 * 1024)
        )));
    }

    let file_path = file.to_path_buf();

    // Wrap in catch_unwind to handle potential panics
    match catch_unwind(AssertUnwindSafe(move || {
        extract_pdf_text_inner(&file_path)
    })) {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(ChunkError::PdfProcessing(
            "PDF parser panicked (corrupted or unsupported format)".to_string()
        )),
    }
}

/// Inner PDF text extraction using lopdf
/// Referred to example from lopdf repo:
/// https://github.com/J-F-Liu/lopdf/blob/main/examples/extract_text.rs
fn extract_pdf_text_inner(file_path: &Path) -> Result<String, ChunkError> {
    use lopdf::Document;

    let doc = Document::load(file_path)
        .map_err(|e| ChunkError::PdfProcessing(format!("Failed to load PDF: {}", e)))?;

    let mut text = String::new();
    let pages = doc.get_pages();

    for (page_num, _) in pages.iter() {
        if let Ok(page_text) = doc.extract_text(&[*page_num]) {
            text.push_str(&page_text);
            text.push('\n');
        }
    }

    if text.trim().is_empty() {
        return Err(ChunkError::PdfProcessing(
            "PDF contains no extractable text (might be scanned/image-only)".to_string()
        ));
    }

    Ok(text)
}

/// Read a text file as UTF-8
fn read_text_file(file: &Path) -> Result<String, ChunkError> {
    fs::read_to_string(file).map_err(|e| {
        if e.kind() == std::io::ErrorKind::InvalidData {
            ChunkError::InvalidUtf8(file.display().to_string())
        } else {
            ChunkError::Io(e)
        }
    })
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
#[derive(Debug, Clone)]
pub struct Chunk {
    pub text: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone)]
pub struct ScoredChunk {
    pub score: f64,
    pub indices: Option<Vec<usize>>,
    pub chunk: Chunk,
}

#[derive(Debug, Clone)]
pub struct FileScore {
    pub path: PathBuf,
    pub score: f64,
    pub top_chunks: Vec<ScoredChunk>,
    pub analysis_duration: Option<std::time::Duration>,
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
