use crate::config::Config;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

#[derive(Default)]
pub struct ThemeSelectorComponent;

impl ThemeSelectorComponent {
    pub fn new() -> Self {
        Self
    }

    /// Render the theme list
    /// `selected_index`: 0 = *Live*, 1+ = theme from list (1-indexed into themes)
    pub fn render(&self, f: &mut Frame, area: Rect, _config: &Config, selected_index: usize) {
        // Get all available themes dynamically
        let available_themes = crate::ui::themes::ThemePresets::list_available_themes();

        // Build theme list - *Live* is index 0, themes are 1+
        let mut items: Vec<ListItem> = Vec::new();

        // *Live* entry (index 0)
        let live_selected = selected_index == 0;
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                if live_selected { "[✓] " } else { "[ ] " },
                Style::default().fg(if live_selected { Color::Green } else { Color::DarkGray }),
            ),
            Span::styled("*Live*", Style::default().fg(Color::Cyan)),
        ])));

        // Theme entries (index 1+)
        for (i, theme) in available_themes.iter().enumerate() {
            let is_selected = selected_index == i + 1;
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    if is_selected { "[✓] " } else { "[ ] " },
                    Style::default().fg(if is_selected { Color::Green } else { Color::DarkGray }),
                ),
                Span::raw(theme.as_str()),
            ])));
        }

        let theme_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Themes"));
        f.render_widget(theme_list, area);
    }
}
