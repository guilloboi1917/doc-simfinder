use crate::{analysis::FileScore, config::Config};
use colored::*;
use std::collections::HashSet;
use std::fmt::Write;

// Helper to format a snippet with highlighted indices.
// If indices are provided, matched character positions are underlined and bold yellow.
// The snippet is padded around the match range and wrapped to terminal width.
fn format_snippet_with_highlights(
    full_text: &str,
    indices: &Option<Vec<usize>>,
    max_snippet_len: usize,
) -> String {
    match indices {
        Some(idx_vec) if !idx_vec.is_empty() => {
            let min_idx = *idx_vec.iter().min().unwrap();
            let max_idx = *idx_vec.iter().max().unwrap();

            // Pad around the match: 60 chars before, or to start if not enough
            let snippet_start = if min_idx > 60 { min_idx - 60 } else { 0 };

            // Extend end to capture match range + padding, up to max_snippet_len
            let snippet_end = (snippet_start + max_snippet_len).min(full_text.len());
            // Ensure we capture at least to the max_idx
            let snippet_end = snippet_end.max(max_idx + 1).min(full_text.len());

            let snippet_sub = &full_text[snippet_start..snippet_end.min(full_text.len())];

            // First, wrap the plain text to terminal width BEFORE coloring
            let width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
            let wrapped_plain = textwrap::fill(snippet_sub, width);

            // Map global indices to positions in the wrapped snippet
            let highlight_indices: HashSet<_> = idx_vec
                .iter()
                .filter_map(|&i| {
                    if i >= snippet_start && i < snippet_end {
                        Some(i - snippet_start)
                    } else {
                        None
                    }
                })
                .collect();

            // Build colored snippet from the wrapped text
            let mut colored = String::new();
            for (i, ch) in wrapped_plain.char_indices() {
                if highlight_indices.contains(&i) {
                    write!(colored, "{}", ch.to_string().underline().bold().yellow()).ok();
                } else {
                    colored.push(ch);
                }
            }

            colored
        }
        _ => {
            // No indices or empty: just return first line, truncated
            let first_line = full_text.lines().next().unwrap_or("");
            let trimmed = first_line.trim();
            if trimmed.len() > 300 {
                format!("{}...", &trimmed[..300])
            } else {
                trimmed.to_string()
            }
        }
    }
}

// Presentation helpers for CLI output with colored indices and scores.
// This returns an ANSI-colored string; callers that need plain text
// can strip ANSI codes.
pub fn present_file_score(score: &FileScore, _config: &Config) -> String {
    let mut out = String::new();

    let file_header = format!("File: {}", score.path.display()).bold().cyan();
    let _ = writeln!(out, "{}", file_header);

    let analysis_duration = match score.analysis_duration {
        Some(duration) => format!("Analysis duration: {:?}", duration).red().italic(),
        None => format!("--").red(),
    };

    let _ = writeln!(out, "{}", analysis_duration);

    let score_str = format!("Score: {:.4}", score.score);
    let score_colored = if score.score > 0.0 {
        score_str.bold().green()
    } else {
        score_str.dimmed()
    };
    let _ = writeln!(out, "{}\n", score_colored);

    if score.top_chunks.is_empty() {
        let _ = writeln!(out, "{}", "No top chunks found.".yellow());
        return out;
    }

    let _ = writeln!(out, "{}", "Top chunks:".bold());
    for (i, c) in score.top_chunks.iter().enumerate() {
        // Add separator before each chunk (except the first one)
        if i > 0 {
            let _ = writeln!(out, "");
            let _ = writeln!(out, "{}", "─".repeat(80).dimmed());
            let _ = writeln!(out, "");
        }

        let idx = format!("{}.", i + 1).bold().yellow();
        let sc = format!("{:.4}", c.score).magenta();
        let range = format!("[{}..{}]", c.chunk.start_byte, c.chunk.end_byte).dimmed();
        let context_header = format!("Context:").underline();

        let formatted_snippet = format_snippet_with_highlights(&c.chunk.text, &c.indices, 300);

        let _ = writeln!(out, "  {} score: {} {}", idx, sc, range);
        let _ = writeln!(out, "{}", context_header);
        let _ = writeln!(out, "     {}", formatted_snippet);
    }

    out
}
