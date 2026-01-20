use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Menu item for save menu
struct SaveMenuItem {
    title: &'static str,
    description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveAction {
    SaveLive,
    SaveAsNewTheme,
}

#[derive(Debug, Clone)]
pub struct SaveMenuComponent {
    pub is_open: bool,
    selected: usize,
}

impl Default for SaveMenuComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveMenuComponent {
    pub fn new() -> Self {
        Self {
            is_open: false,
            selected: 0,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.selected = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn move_selection(&mut self, delta: i32) {
        let items = self.get_menu_items();
        let new_sel = self.selected as i32 + delta;
        if new_sel >= 0 && new_sel < items.len() as i32 {
            self.selected = new_sel as usize;
        }
    }

    pub fn get_selected_action(&self) -> SaveAction {
        match self.selected {
            0 => SaveAction::SaveLive,
            1 => SaveAction::SaveAsNewTheme,
            _ => SaveAction::SaveLive,
        }
    }

    fn get_menu_items(&self) -> Vec<SaveMenuItem> {
        vec![
            SaveMenuItem {
                title: "Overwrite *Live*",
                description: "Save current settings to config.toml",
            },
            SaveMenuItem {
                title: "Save as named Theme",
                description: "Create a new theme file in themes/ directory",
            },
        ]
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.is_open {
            return;
        }

        // Calculate popup dimensions (smaller now with only 2 cards)
        let popup_width = 60_u16.min(area.width.saturating_sub(4));
        let popup_height = 14_u16;

        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        // Clear and draw background
        f.render_widget(Clear, popup_area);
        let bg = Block::default()
            .style(Style::default().bg(Color::Rgb(20, 20, 30)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                " Save ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        f.render_widget(bg, popup_area);

        let inner = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        // Layout: 2 cards + footer
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Card 1
                Constraint::Length(1), // Spacer
                Constraint::Length(4), // Card 2
                Constraint::Min(1),    // Footer
            ])
            .split(inner);

        let menu_items = self.get_menu_items();
        let card_indices = [0, 2]; // Layout indices for cards

        for (i, item) in menu_items.iter().enumerate() {
            let is_selected = i == self.selected;

            let (border_color, title_color, bg_color) = if is_selected {
                (Color::Cyan, Color::Cyan, Color::Rgb(40, 40, 60))
            } else {
                (Color::DarkGray, Color::White, Color::Rgb(30, 30, 45))
            };

            // Diagonal selector chars
            let (sel1, sel2) = if is_selected {
                (
                    Span::styled("╲ ", Style::default().fg(Color::Yellow)),
                    Span::styled("╱ ", Style::default().fg(Color::Yellow)),
                )
            } else {
                (
                    Span::styled("  ", Style::default()),
                    Span::styled("  ", Style::default()),
                )
            };

            let card_content = Text::from(vec![
                Line::from(vec![
                    sel1,
                    Span::styled(
                        item.title,
                        Style::default()
                            .fg(title_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    sel2,
                    Span::styled(item.description, Style::default().fg(Color::Gray)),
                ]),
            ]);

            let card = Paragraph::new(card_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                        .style(Style::default().bg(bg_color)),
                )
                .alignment(Alignment::Left);

            f.render_widget(card, layout[card_indices[i]]);
        }

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("[↑↓]", Style::default().fg(Color::Yellow)),
            Span::styled(" Move  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::styled(" Confirm  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
        ]))
        .alignment(Alignment::Center);

        f.render_widget(footer, layout[3]);
    }
}
