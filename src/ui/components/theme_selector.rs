use crate::config::Config;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Default)]
pub struct ThemeSelectorComponent;

impl ThemeSelectorComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut Frame, area: Rect, config: &Config) {
        // Get all available themes dynamically
        let available_themes = crate::ui::themes::ThemePresets::list_available_themes();

        // Calculate available width (minus borders and spacing)
        let content_width = area.width.saturating_sub(2); // Remove borders

        // Build theme options with auto-wrapping
        // *Live* is always first and always selected (it's what we're editing)
        let mut lines = Vec::new();
        let mut current_line = String::from("[✓] *Live*");

        for theme in available_themes.iter() {
            let theme_part = format!("[ ] {}", theme);
            let part_with_sep = format!("  {}", theme_part);

            // Check if this part fits in current line
            let would_fit = current_line.len() + part_with_sep.len() <= content_width as usize;

            if would_fit {
                current_line.push_str(&part_with_sep);
            } else {
                // Start new line
                lines.push(current_line);
                current_line = theme_part; // No indent for continuation lines
            }
        }

        if !current_line.trim().is_empty() {
            lines.push(current_line);
        }

        // Add separator display at the end
        let separator_display = format!("\nSeparator: \"{}\"", config.style.separator);

        let full_text = format!("{}{}", lines.join("\n"), separator_display);
        let theme_selector = Paragraph::new(full_text)
            .block(Block::default().borders(Borders::ALL).title("Themes"))
            .wrap(ratatui::widgets::Wrap { trim: false });
        f.render_widget(theme_selector, area);
    }
}
