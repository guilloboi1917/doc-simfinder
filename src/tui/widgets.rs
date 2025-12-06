// Widget implementations for TUI
//
// See docs/copilot/ui.md for widget patterns

use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, Padding, Paragraph},
};

use super::focus::{Focus, FocusManager};
use super::layout::{LayoutConfig, results_two_column, right_panel_split};
use crate::analysis::FileScore;
use crate::state_machine::AppState;

/// Helper to build highlighted text lines with matched character indices.
/// Returns a vector of Lines with proper highlighting and text wrapping.
/// Matched characters are styled with yellow, bold, and underline.
fn build_highlighted_lines(
    text: &str,
    indices: &Option<Vec<usize>>,
    max_width: usize,
) -> Vec<Line<'static>> {
    // First, wrap the text to prevent overflow
    let wrapped = textwrap::wrap(text, max_width.saturating_sub(2)); // -2 for padding

    let mut result_lines = Vec::new();
    let mut char_offset = 0;

    for wrapped_line in wrapped {
        let line_text = wrapped_line.to_string();

        // Build spans for this line with highlighting
        let spans = match indices {
            Some(idx_vec) if !idx_vec.is_empty() => {
                let mut spans = Vec::new();
                let mut current_text = String::new();
                let mut is_highlighted = false;

                // Find the actual position in original text for each character in wrapped line
                for ch in line_text.chars() {
                    // Find this character at or after char_offset in the original text
                    let mut found_at = None;
                    for (idx, orig_ch) in text[char_offset..].char_indices() {
                        if orig_ch == ch {
                            found_at = Some(char_offset + idx);
                            break;
                        }
                    }

                    let global_i = found_at.unwrap_or(char_offset);
                    let should_highlight = idx_vec.contains(&global_i);

                    if should_highlight != is_highlighted {
                        // Flush current span if it has content
                        if !current_text.is_empty() {
                            let span = if is_highlighted {
                                Span::styled(
                                    current_text.clone(),
                                    Style::default()
                                        .fg(Color::Yellow)
                                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                                )
                            } else {
                                Span::raw(current_text.clone())
                            };
                            spans.push(span);
                            current_text.clear();
                        }
                        is_highlighted = should_highlight;
                    }

                    current_text.push(ch);

                    // Move char_offset forward to the next character position
                    if let Some(idx) = found_at {
                        char_offset = idx + ch.len_utf8();
                    }
                }

                // Flush remaining text
                if !current_text.is_empty() {
                    let span = if is_highlighted {
                        Span::styled(
                            current_text,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        )
                    } else {
                        Span::raw(current_text)
                    };
                    spans.push(span);
                }

                spans
            }
            _ => {
                // No indices - return plain text
                vec![Span::raw(line_text.clone())]
            }
        };

        result_lines.push(Line::from(spans));

        // Skip any whitespace between lines in the original text
        while char_offset < text.len() {
            if let Some(ch) = text[char_offset..].chars().next() {
                if ch.is_whitespace() && ch != '\n' {
                    char_offset += ch.len_utf8();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    if result_lines.is_empty() {
        result_lines.push(Line::from(text.to_string()));
    }

    result_lines
}

/// Dashboard - main widget orchestrator
pub struct Dashboard {
    layout: LayoutConfig,
}

impl Dashboard {
    /// Create a new dashboard for the given state
    pub fn new_for_state(state: &AppState) -> Self {
        Self {
            layout: LayoutConfig::for_state(state),
        }
    }

    /// Render the dashboard
    pub fn render(&self, frame: &mut Frame, state: &AppState, focus: &FocusManager) {
        match state {
            AppState::Configuring {
                config,
                validation_errors,
                walk_result,
                autocomplete_available,
                autocomplete_suggestion,
            } => {
                self.render_configuring(
                    frame,
                    config,
                    validation_errors,
                    walk_result,
                    autocomplete_available,
                    autocomplete_suggestion,
                    focus,
                );
            }
            AppState::ViewingResults {
                results,
                selected_index,
                total_duration,
                ..
            } => {
                self.render_results(frame, results, *selected_index, focus, *total_duration);
            }
            AppState::ViewingFileDetail {
                file_result,
                scroll_position,
                ..
            } => {
                self.render_file_detail(frame, file_result, *scroll_position, focus);
            }
            AppState::Analyzing {
                files_processed,
                total_files,
                query,
                ..
            } => {
                self.render_analyzing(frame, *files_processed, *total_files, query);
            }
            AppState::Error { message, .. } => {
                self.render_error(frame, message);
            }
            AppState::Exiting => {
                // Blank screen or exit message
            }
        }
    }

    fn render_configuring(
        &self,
        frame: &mut Frame,
        config: &crate::config::Config,
        validation_errors: &[String],
        walk_result: &Option<crate::file_walker::WalkResult>,
        autocomplete_available: &bool,
        autocomplete_suggestion: &Option<String>,
        focus: &FocusManager,
    ) {
        let chunks = self.layout.split(frame.area());

        // Path input - render directly from config
        if let Some(&area) = chunks.get(0) {
            let is_focused = focus.is_focused(Focus::PathInput);
            let displayed_path: Text = if *autocomplete_available {
                if let Some(suggestion) = autocomplete_suggestion {
                    let current_path = config.search_path.to_string_lossy();
                    // Only show suffix if suggestion is longer than current path
                    if suggestion.len() > current_path.len() {
                        let suggestion_suffix = &suggestion[current_path.len()..];
                        Text::from(Line::from(vec![
                            Span::raw(current_path.clone()),
                            Span::styled(
                                suggestion_suffix,
                                Style::default()
                                    .fg(Color::DarkGray)
                                    .add_modifier(Modifier::ITALIC),
                            ),
                        ]))
                    } else {
                        Text::from(current_path.to_string())
                    }
                } else {
                    Text::from(config.search_path.to_string_lossy().to_string())
                }
            } else {
                Text::from(config.search_path.to_string_lossy().to_string())
            };
            let path_widget = Paragraph::new(displayed_path)
                .style(if config.search_path.exists() {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Search Path (editable) ")
                        .border_type(if is_focused {
                            BorderType::Double
                        } else {
                            BorderType::Plain
                        }),
                );
            if is_focused {
                frame.set_cursor_position(Position::new(
                    area.x + config.search_path.to_string_lossy().len() as u16 + 1,
                    area.y + 1,
                ));
            }

            frame.render_widget(path_widget, area);
        }

        // Query input - render directly from config
        if let Some(&area) = chunks.get(1) {
            let is_focused = focus.is_focused(Focus::QueryInput);
            let query_widget = Paragraph::new(config.query.as_str())
                .style(if !config.query.is_empty() {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Query (editable) ")
                        .border_type(if is_focused {
                            BorderType::Double
                        } else {
                            BorderType::Plain
                        }),
                );
            if is_focused {
                frame.set_cursor_position(Position::new(
                    area.x + config.query.len() as u16 + 1,
                    area.y + 1,
                ));
            }
            frame.render_widget(query_widget, area);
        }

        // File list - show found files from walk_result
        if let Some(&area) = chunks.get(2) {
            let is_focused = focus.is_focused(Focus::FileList);

            if let Some(walk_result) = walk_result {
                // Display list of found files
                let items: Vec<ListItem> = walk_result
                    .files
                    .iter()
                    .map(|path| {
                        // Normalize path separators for consistency
                        let normalized = path.display().to_string().replace('\\', "/");
                        ListItem::new(normalized)
                    })
                    .collect();

                let title = format!(" Found Files ({}) ", walk_result.files.len());
                let file_list = List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .border_type(if is_focused {
                            BorderType::Double
                        } else {
                            BorderType::Plain
                        }),
                );
                frame.render_widget(file_list, area);
            } else {
                // Show placeholder when no walk result yet
                let placeholder = Paragraph::new("Searching for files...")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Found Files ")
                            .border_type(if is_focused {
                                BorderType::Double
                            } else {
                                BorderType::Plain
                            }),
                    );
                frame.render_widget(placeholder, area);
            }
        }

        // Options panel (read-only display) or validation errors
        if let Some(&area) = chunks.get(3) {
            let is_focused = focus.is_focused(Focus::OptionsPanel);

            // Show validation errors OR options
            if !validation_errors.is_empty() {
                let error_lines: Vec<Line> = validation_errors
                    .iter()
                    .map(|e| Line::from(Span::styled(e.clone(), Style::default().fg(Color::Red))))
                    .collect();
                let error_widget = Paragraph::new(error_lines).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Validation Errors ")
                        .border_style(Style::default().fg(Color::Red)),
                );
                frame.render_widget(error_widget, area);
            } else {
                let options_text = format!(
                    "- Window Size: {:<15}\n- Max Window: {:<15}\n- Threshold: {:<15.2}\n- Top N: {:<15}\n- Threads: {:<15}\n- File Exts: {:<15}",
                    config.window_size,
                    config.max_window_size,
                    config.threshold,
                    config.top_n,
                    if config.num_threads > 0 {
                        config.num_threads.to_string()
                    } else {
                        "All".into()
                    },
                    config.file_exts.join(", ")
                );
                let options_widget = Paragraph::new(options_text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Options (Read-Only) ")
                        .padding(Padding::horizontal(1))
                        .border_type(if is_focused {
                            BorderType::Double
                        } else {
                            BorderType::Plain
                        }),
                );
                frame.render_widget(options_widget, area);
            }
        }

        // Start button
        if let Some(&area) = chunks.get(4) {
            let is_focused = focus.is_focused(Focus::StartButton);
            let can_start = !config.query.is_empty()
                && config.search_path.exists()
                && walk_result.is_some()
                && walk_result
                    .as_ref()
                    .map(|wr| !wr.files.is_empty())
                    .unwrap_or(false);

            let (button_text, button_style) = if can_start {
                (
                    "✓ Ready to Start Analysis",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    "⚠ Configure query and valid path first",
                    Style::default().fg(Color::DarkGray),
                )
            };

            let start_widget = Paragraph::new(button_text).style(button_style).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Start ")
                    .border_type(if is_focused {
                        BorderType::Double
                    } else {
                        BorderType::Plain
                    })
                    .title_bottom(Line::from(" <Enter> to Start Analysis ").centered()),
            );
            frame.render_widget(start_widget, area);
        }
    }

    fn render_results(
        &self,
        frame: &mut Frame,
        results: &[FileScore],
        selected_index: usize,
        focus: &FocusManager,
        total_duration: Option<std::time::Duration>,
    ) {
        let (left, right) = results_two_column(frame.area());

        // File list (left)
        let is_focused = focus.is_focused(Focus::FileList);
        let items: Vec<ListItem> = results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let prefix = if i == selected_index { "▶ " } else { "  " };
                let style = if is_focused && i == selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // Normalize path separators to forward slashes for consistency
                let normalized_path = result.path.display().to_string().replace('\\', "/");
                let text = format!("{}{}", prefix, normalized_path);
                ListItem::new(text).style(style)
            })
            .collect();

        let file_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Files")
                    .border_style(if is_focused {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    })
                    .title_bottom(
                        Line::from(" <↑↓> | <jk> to navigate, <Enter> to view file details ")
                            .centered(),
                    ),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        // Create stateful list with selection to enable scrolling
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(selected_index));

        frame.render_stateful_widget(file_list, left, &mut list_state);

        // Right panel (preview, stats, actions)
        let (preview_area, stats_area, actions_area) = right_panel_split(right);

        // Preview
        if let Some(selected) = results.get(selected_index) {
            let preview_focused = focus.is_focused(Focus::FilePreview);
            self.render_file_preview(frame, selected, preview_area, preview_focused);
        }

        // Stats
        self.render_stats(frame, results, stats_area, total_duration);

        // Actions
        self.render_actions(frame, actions_area);
    }

    fn render_file_preview(
        &self,
        frame: &mut Frame,
        file_result: &FileScore,
        area: Rect,
        is_focused: bool,
    ) {
        let mut lines = vec![];

        for (i, chunk) in file_result.top_chunks.iter().take(3).enumerate() {
            // Add separator before each chunk (except the first one)
            if i > 0 {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "─".repeat(40),
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(""));
            }

            // Match header with colored index and score
            let match_line = Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{}.", i + 1),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" score: "),
                Span::styled(
                    format!("{:.4}", chunk.score),
                    Style::default().fg(Color::Magenta),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("[{}..{}]", chunk.chunk.start_byte, chunk.chunk.end_byte),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            lines.push(match_line);

            // Context header
            lines.push(Line::from(Span::styled(
                "Context:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));

            // Context text with character-level highlighting and proper wrapping
            // Calculate available width (subtract borders and padding)
            let available_width = area.width.saturating_sub(4).max(40) as usize;
            let context_lines =
                build_highlighted_lines(&chunk.chunk.text, &chunk.indices, available_width);
            for ctx_line in context_lines.iter().take(3) {
                // Limit lines in preview
                lines.push(ctx_line.clone());
            }
            if context_lines.len() > 3 {
                lines.push(Line::from(Span::styled(
                    "...",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            lines.push(Line::from(""));
        }

        let preview = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Preview")
                .border_style(if is_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        );
        frame.render_widget(preview, area);
    }

    fn render_stats(
        &self,
        frame: &mut Frame,
        results: &[FileScore],
        area: Rect,
        total_duration: Option<std::time::Duration>,
    ) {
        let matched = results.iter().filter(|r| r.score > 0.0).count();

        let duration_text = if let Some(duration) = total_duration {
            format!("{:.2}s", duration.as_secs_f64())
        } else {
            "N/A".to_string()
        };

        let lines = vec![
            Line::from(format!("Total files: {}", results.len())),
            Line::from(format!("Matches: {}", matched)),
            Line::from(format!("Duration: {}", duration_text)),
        ];

        let stats =
            Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Statistics"));
        frame.render_widget(stats, area);
    }

    fn render_actions(&self, frame: &mut Frame, area: Rect) {
        let lines = vec![
            Line::from("Ctrl+O: Open Location"),
            Line::from("Ctrl+R: Reanalyze"),
            Line::from("Esc: Back"),
            Line::from("Ctrl+Q: Quit"),
        ];

        let actions =
            Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Actions"));
        frame.render_widget(actions, area);
    }

    fn render_file_detail(
        &self,
        frame: &mut Frame,
        file_result: &FileScore,
        scroll_position: usize,
        _focus: &FocusManager,
    ) {
        let chunks = self.layout.split(frame.area());

        if let Some(&area) = chunks.get(0) {
            let mut lines = vec![Line::from(Span::styled(
                format!("File: {}", file_result.path.display()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))];

            // Analysis duration
            if let Some(duration) = file_result.analysis_duration {
                lines.push(Line::from(Span::styled(
                    format!("Analysis duration: {:?}", duration),
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::ITALIC),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "--",
                    Style::default().fg(Color::Red),
                )));
            }

            // Score with conditional coloring
            let score_style = if file_result.score > 0.0 {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(Span::styled(
                format!("Score: {:.4}", file_result.score),
                score_style,
            )));
            lines.push(Line::from(""));

            if file_result.top_chunks.is_empty() {
                lines.push(Line::from(Span::styled(
                    "No top chunks found.",
                    Style::default().fg(Color::Yellow),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "Top chunks:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));

                for (i, chunk) in file_result.top_chunks.iter().enumerate() {
                    // Add separator before each chunk (except the first one)
                    if i > 0 {
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            "─".repeat(80),
                            Style::default().fg(Color::DarkGray),
                        )));
                        lines.push(Line::from(""));
                    }

                    // Match header with colored index and score
                    let match_line = Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            format!("{}.", i + 1),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" score: "),
                        Span::styled(
                            format!("{:.4}", chunk.score),
                            Style::default().fg(Color::Magenta),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            format!("[{}..{}]", chunk.chunk.start_byte, chunk.chunk.end_byte),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]);
                    lines.push(match_line);

                    // Context header
                    lines.push(Line::from(Span::styled(
                        "Context:",
                        Style::default().add_modifier(Modifier::UNDERLINED),
                    )));

                    // Full chunk text with character-level highlighting and proper wrapping
                    // Calculate available width (subtract borders and padding: 2 borders + 2 horizontal padding)
                    let available_width = area.width.saturating_sub(4).max(40) as usize;
                    let context_lines =
                        build_highlighted_lines(&chunk.chunk.text, &chunk.indices, available_width);
                    for ctx_line in context_lines {
                        lines.push(ctx_line);
                    }
                }
            }

            let content = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("File Detail")
                        .padding(ratatui::widgets::Padding::horizontal(1))
                        .title_bottom(
                            Line::from(" <↑↓> | <jk> to navigate, <Esc> to return to overview ")
                                .centered(),
                        ), // Add 1 char padding on left/right
                )
                .scroll((scroll_position as u16, 0));
            frame.render_widget(content, area);
        }
    }

    fn render_analyzing(
        &self,
        frame: &mut Frame,
        files_processed: usize,
        total_files: usize,
        query: &str,
    ) {
        let area = frame.area();
        let progress_text = if total_files > 0 {
            format!(
                "Analyzing: {} / {} files ({}%)\nQuery: {}",
                files_processed,
                total_files,
                (files_processed * 100) / total_files,
                query
            )
        } else {
            format!("Discovering files...\nQuery: {}", query)
        };

        let widget = Paragraph::new(progress_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Analysis in Progress"),
        );
        frame.render_widget(widget, area);
    }

    fn render_error(&self, frame: &mut Frame, message: &str) {
        let area = frame.area();
        let error = Paragraph::new(message)
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Error")
                    .border_style(Style::default().fg(Color::Red)),
            );
        frame.render_widget(error, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let config = crate::config::Config::default();
        let state = AppState::Configuring {
            config,
            validation_errors: vec![],
            walk_result: None,
            autocomplete_available: false,
            autocomplete_suggestion: None,
        };
        let _dashboard = Dashboard::new_for_state(&state);
        // Test that dashboard was created
    }
}
