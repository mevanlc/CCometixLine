use crate::config::{Config, SegmentId};
use crate::core::segments::SegmentData;
use crate::core::StatusLineGenerator;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

pub struct PreviewComponent {
    preview_cache: String,
    statusline_text: Text<'static>,
}

impl Default for PreviewComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewComponent {
    pub fn new() -> Self {
        Self {
            preview_cache: String::new(),
            statusline_text: Text::default(),
        }
    }

    pub fn update_preview(&mut self, config: &Config) {
        self.update_preview_with_width(config, 80); // Default width
    }

    pub fn update_preview_with_width(&mut self, config: &Config, width: u16) {
        // Generate mock segments data directly for preview
        let segments_data = self.generate_mock_segments_data(config);

        // Generate both string and TUI text versions
        let renderer = StatusLineGenerator::new(config.clone());

        // Keep string version for compatibility (if needed elsewhere)
        self.preview_cache = renderer.generate(segments_data.clone());

        // Generate TUI-optimized text with smart segment wrapping for preview display
        // Use actual available width minus some padding for the simulated prompt
        let content_width = width.saturating_sub(4);
        let preview_result = renderer.generate_for_tui_preview(segments_data, content_width);

        // Convert to owned text by cloning the spans
        let owned_lines: Vec<Line<'static>> = preview_result
            .lines
            .into_iter()
            .map(|line| {
                let owned_spans: Vec<ratatui::text::Span<'static>> = line
                    .spans
                    .into_iter()
                    .map(|span| ratatui::text::Span::styled(span.content.to_string(), span.style))
                    .collect();
                Line::from(owned_spans)
            })
            .collect();

        self.statusline_text = Text::from(owned_lines);
    }

    pub fn calculate_height(&self) -> u16 {
        // Fixed height for the Claude Code simulation:
        // 10 lines for header box + 1 blank + statusline lines + 2 blank
        let statusline_lines = self.statusline_text.lines.len().max(1);
        (13 + statusline_lines) as u16
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Split area: header box (10 lines) + blank + statusline + 2 blanks
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Header box
                Constraint::Length(1),  // Blank line
                Constraint::Min(1),     // Statusline + 2 blank lines
            ])
            .split(area);

        // Header box content (centered text)
        let header_content = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("▐▛███▜▌", Style::default().fg(Color::Rgb(204, 120, 50))),
            ]),
            Line::from(vec![
                Span::styled("▝▜█████▛▘", Style::default().fg(Color::Rgb(204, 120, 50))),
            ]),
            Line::from(vec![
                Span::styled("  ▘▘ ▝▝", Style::default().fg(Color::Rgb(204, 120, 50))),
            ]),
            Line::from(vec![
                Span::styled("  Opus · Sonnet · Haiku  ", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("you@example.com", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled(format!("~/CCometixLine-v{}", env!("CARGO_PKG_VERSION")), Style::default().fg(Color::Cyan)),
            ]),
        ]);

        let header_box = Paragraph::new(header_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(border::ROUNDED)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" Claude Code v1.0.31 ")
                    .title_style(Style::default().fg(Color::DarkGray)),
            )
            .alignment(Alignment::Center);
        f.render_widget(header_box, layout[0]);

        // Blank line (layout[1]) - nothing to render

        // Statusline area with 2 trailing blank lines
        let mut statusline_lines: Vec<Line<'static>> = Vec::new();
        for line in self.statusline_text.lines.iter() {
            let mut spans = vec![Span::raw("  ")];
            spans.extend(line.spans.iter().cloned());
            statusline_lines.push(Line::from(spans));
        }
        statusline_lines.push(Line::from("")); // Blank line 1
        statusline_lines.push(Line::from("")); // Blank line 2

        let statusline = Paragraph::new(Text::from(statusline_lines));
        f.render_widget(statusline, layout[2]);
    }

    pub fn get_preview_cache(&self) -> &str {
        &self.preview_cache
    }

    /// Generate mock segments data for preview display
    /// This creates perfect preview data without depending on real environment
    fn generate_mock_segments_data(
        &self,
        config: &Config,
    ) -> Vec<(crate::config::SegmentConfig, SegmentData)> {
        let mut segments_data = Vec::new();

        for segment_config in &config.segments {
            if !segment_config.enabled {
                continue;
            }

            let mock_data = match segment_config.id {
                SegmentId::Model => SegmentData {
                    primary: "Sonnet 4".to_string(),
                    secondary: "".to_string(),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("model".to_string(), "claude-4-sonnet-20250512".to_string());
                        map
                    },
                },
                SegmentId::Directory => SegmentData {
                    primary: "CCometixLine".to_string(),
                    secondary: "".to_string(),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("current_dir".to_string(), "~/CCometixLine".to_string());
                        map
                    },
                },
                SegmentId::Git => SegmentData {
                    primary: "master".to_string(),
                    secondary: "✓".to_string(),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("branch".to_string(), "master".to_string());
                        map.insert("status".to_string(), "Clean".to_string());
                        map.insert("ahead".to_string(), "0".to_string());
                        map.insert("behind".to_string(), "0".to_string());
                        map
                    },
                },
                SegmentId::ContextWindow => SegmentData {
                    primary: "78.2%".to_string(),
                    secondary: "· 156.4k".to_string(),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("total_tokens".to_string(), "156400".to_string());
                        map.insert("percentage".to_string(), "78.2".to_string());
                        map.insert("session_tokens".to_string(), "48200".to_string());
                        map
                    },
                },
                SegmentId::Usage => SegmentData {
                    primary: "24%".to_string(),
                    secondary: "· 10-7-2".to_string(),
                    metadata: HashMap::new(),
                },
                SegmentId::Cost => SegmentData {
                    primary: "$0.02".to_string(),
                    secondary: "".to_string(),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("cost".to_string(), "0.01234".to_string());
                        map
                    },
                },
                SegmentId::Session => SegmentData {
                    primary: "3m45s".to_string(),
                    secondary: "+156 -23".to_string(),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("duration_ms".to_string(), "225000".to_string());
                        map.insert("lines_added".to_string(), "156".to_string());
                        map.insert("lines_removed".to_string(), "23".to_string());
                        map
                    },
                },
                SegmentId::OutputStyle => SegmentData {
                    primary: "default".to_string(),
                    secondary: "".to_string(),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("style_name".to_string(), "default".to_string());
                        map
                    },
                },
                SegmentId::Update => SegmentData {
                    primary: format!("v{}", env!("CARGO_PKG_VERSION")),
                    secondary: "".to_string(),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert(
                            "current_version".to_string(),
                            env!("CARGO_PKG_VERSION").to_string(),
                        );
                        map.insert("update_available".to_string(), "false".to_string());
                        map
                    },
                },
            };

            segments_data.push((segment_config.clone(), mock_data));
        }

        segments_data
    }
}
