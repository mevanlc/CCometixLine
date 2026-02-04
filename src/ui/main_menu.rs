use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

#[derive(Default)]
pub struct MainMenu {
    selected_item: usize,
    should_quit: bool,
    show_about: bool,
    status_message: Option<StatusMessage>,
}

/// Status message to display in the footer
struct StatusMessage {
    message: String,
    is_error: bool,
}

/// Menu item with title and description
struct MenuItem {
    title: String,
    description: String,
    compact: bool, // If true, render as single line
}

#[derive(Debug)]
pub enum MenuResult {
    LaunchConfigurator,
    InstallBinary,
    CheckConfig,
    Exit,
}

impl MainMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run() -> Result<Option<MenuResult>, Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut app = MainMenu::new();
        let result = app.main_loop(&mut terminal)?;

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(result)
    }

    fn main_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<Option<MenuResult>, Box<dyn std::error::Error>> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if self.show_about {
                    // In about dialog, any key closes it
                    self.show_about = false;
                    continue;
                }

                // Clear status message on any key press
                self.status_message = None;

                // Check for cancel (Esc, q, or Ctrl+C)
                let is_cancel = key.code == KeyCode::Esc
                    || key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL));

                if is_cancel {
                    self.should_quit = true;
                }

                match key.code {
                    KeyCode::Up => {
                        if self.selected_item > 0 {
                            self.selected_item -= 1;
                        }
                    }
                    KeyCode::Down => {
                        let menu_items = self.get_menu_items();
                        if self.selected_item < menu_items.len() - 1 {
                            self.selected_item += 1;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(result) = self.handle_selection() {
                            return Ok(Some(result));
                        }
                        // None means stay in menu (action was handled internally)
                    }
                    _ => {}
                }
            }

            if self.should_quit {
                return Ok(Some(MenuResult::Exit));
            }
        }
    }

    fn get_install_target_path() -> Option<std::path::PathBuf> {
        dirs::home_dir().map(|home| home.join(".claude").join("ccline").join("ccline"))
    }

    fn is_binary_installed() -> bool {
        Self::get_install_target_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    fn get_menu_items(&self) -> Vec<MenuItem> {
        let (install_title, install_desc) = if Self::is_binary_installed() {
            (
                "Reinstall ccline binary".to_string(),
                "Replace the installed binary with the currently running version".to_string(),
            )
        } else {
            (
                "Install ccline binary".to_string(),
                "Copy this binary to ~/.claude/ccline/ccline".to_string(),
            )
        };

        vec![
            MenuItem {
                title: "Configuration mode".to_string(),
                description: "Customize icons, colors, info fields, and separators".to_string(),
                compact: false,
            },
            MenuItem {
                title: install_title,
                description: install_desc,
                compact: false,
            },
            MenuItem {
                title: "Validate configuration".to_string(),
                description: "Check your configuration for errors".to_string(),
                compact: false,
            },
            MenuItem {
                title: "About".to_string(),
                description: "Show application information".to_string(),
                compact: true,
            },
            MenuItem {
                title: "Exit".to_string(),
                description: String::new(),
                compact: true,
            },
        ]
    }

    fn handle_selection(&mut self) -> Option<MenuResult> {
        match self.selected_item {
            0 => Some(MenuResult::LaunchConfigurator),
            1 => {
                // Install binary to ~/.claude/ccline/ccline
                self.install_binary();
                None // Stay in menu
            }
            2 => {
                // Check config and show result in footer
                match crate::config::Config::load() {
                    Ok(config) => match config.check() {
                        Ok(_) => {
                            self.status_message = Some(StatusMessage {
                                message: "✓ Configuration is valid!".to_string(),
                                is_error: false,
                            });
                        }
                        Err(e) => {
                            self.status_message = Some(StatusMessage {
                                message: format!("✗ Invalid: {}", e),
                                is_error: true,
                            });
                        }
                    },
                    Err(e) => {
                        self.status_message = Some(StatusMessage {
                            message: format!("✗ Failed to load: {}", e),
                            is_error: true,
                        });
                    }
                }
                None // Stay in menu
            }
            3 => {
                self.show_about = true;
                None // Stay in menu
            }
            4 => Some(MenuResult::Exit),
            _ => Some(MenuResult::Exit),
        }
    }

    fn install_binary(&mut self) {
        // Get current executable path
        let current_exe = match std::env::current_exe() {
            Ok(path) => path,
            Err(e) => {
                self.status_message = Some(StatusMessage {
                    message: format!("✗ Failed to get executable path: {}", e),
                    is_error: true,
                });
                return;
            }
        };

        // Get target path ~/.claude/ccline/ccline
        let target_path = match Self::get_install_target_path() {
            Some(path) => path,
            None => {
                self.status_message = Some(StatusMessage {
                    message: "✗ Failed to determine home directory".to_string(),
                    is_error: true,
                });
                return;
            }
        };
        let target_dir = target_path.parent().unwrap();

        // Create directory if needed
        if let Err(e) = std::fs::create_dir_all(&target_dir) {
            self.status_message = Some(StatusMessage {
                message: format!("✗ Failed to create directory: {}", e),
                is_error: true,
            });
            return;
        }

        // Copy the binary
        match std::fs::copy(&current_exe, &target_path) {
            Ok(_) => {
                // Make executable on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = std::fs::metadata(&target_path) {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o755);
                        let _ = std::fs::set_permissions(&target_path, perms);
                    }
                }
                self.status_message = Some(StatusMessage {
                    message: format!("✓ Installed to {}", target_path.display()),
                    is_error: false,
                });
            }
            Err(e) => {
                self.status_message = Some(StatusMessage {
                    message: format!("✗ Failed to copy binary: {}", e),
                    is_error: true,
                });
            }
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        let size = f.area();

        // Dark background for entire screen
        let bg_block = Block::default().style(Style::default().bg(Color::Rgb(20, 20, 30)));
        f.render_widget(bg_block, size);

        // Calculate footer height based on status message
        let footer_height = if self.status_message.is_some() { 4 } else { 2 };

        // Main layout - vertical
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),             // Header
                Constraint::Min(10),               // Menu cards
                Constraint::Length(footer_height), // Footer/Help
            ])
            .split(size);

        // Center the content horizontally (70% width, centered)
        let center_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ])
            .split(main_layout[1]);

        let card_area = center_layout[1];

        // Header - centered title
        let header_text = Text::from(vec![
            Line::from(vec![
                Span::styled(
                    "CCometixLine",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" v", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    env!("CARGO_PKG_VERSION"),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(Span::styled(
                "High-performance Claude Code StatusLine",
                Style::default().fg(Color::DarkGray),
            )),
        ]);

        let header = Paragraph::new(header_text).alignment(Alignment::Center);
        f.render_widget(header, main_layout[0]);

        // Menu cards
        let menu_items = self.get_menu_items();

        // Calculate card heights: 4 for regular cards (2 content + 2 border), 3 for compact (1 content + 2 border)
        // Add 1-line spacers between cards
        let mut constraints: Vec<Constraint> = Vec::new();
        for (i, item) in menu_items.iter().enumerate() {
            if i > 0 {
                constraints.push(Constraint::Length(1)); // Spacer between cards
            }
            let height = if item.compact { 3 } else { 4 };
            constraints.push(Constraint::Length(height));
        }
        constraints.push(Constraint::Min(0)); // Spacer at bottom

        let card_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(card_area);

        // Render each card (accounting for spacer indices)
        let mut layout_idx = 0;
        for (i, item) in menu_items.iter().enumerate() {
            if i > 0 {
                layout_idx += 1; // Skip spacer
            }

            let is_selected = i == self.selected_item;

            let (border_color, title_color, bg_color) = if is_selected {
                (Color::Cyan, Color::Cyan, Color::Rgb(40, 40, 60))
            } else {
                (Color::DarkGray, Color::White, Color::Rgb(30, 30, 45))
            };

            // Diagonal selector chars for selected item
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

            let card_content = if item.compact {
                // Single line for compact items
                Text::from(vec![
                    Line::from(vec![
                        if is_selected {
                            Span::styled("> ", Style::default().fg(Color::Yellow))
                        } else {
                            Span::styled("  ", Style::default())
                        },
                        Span::styled(
                            item.title.as_str(),
                            Style::default()
                                .fg(title_color)
                                .add_modifier(if is_selected {
                                    Modifier::BOLD
                                } else {
                                    Modifier::empty()
                                }),
                        ),
                    ]),
                ])
            } else {
                // Two lines: title + description with diagonal selectors
                Text::from(vec![
                    Line::from(vec![
                        sel1,
                        Span::styled(
                            item.title.as_str(),
                            Style::default()
                                .fg(title_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        sel2,
                        Span::styled(item.description.as_str(), Style::default().fg(Color::Gray)),
                    ]),
                ])
            };

            let card = Paragraph::new(card_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                        .style(Style::default().bg(bg_color)),
                )
                .alignment(Alignment::Left);

            f.render_widget(card, card_layout[layout_idx]);
            layout_idx += 1;
        }

        // Footer - minimal, centered
        let mut footer_lines = vec![Line::from(vec![
            Span::styled("[↑↓]", Style::default().fg(Color::Yellow)),
            Span::styled(" Navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::styled(" Select  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::styled(" Exit", Style::default().fg(Color::DarkGray)),
        ])];

        // Add status message if present
        if let Some(ref status) = self.status_message {
            let color = if status.is_error {
                Color::Red
            } else {
                Color::Green
            };
            footer_lines.push(Line::from(Span::styled(
                status.message.as_str(),
                Style::default().fg(color),
            )));
        }

        let footer = Paragraph::new(Text::from(footer_lines)).alignment(Alignment::Center);
        f.render_widget(footer, main_layout[2]);

        // About dialog overlay
        if self.show_about {
            self.render_about_dialog(f, size);
        }
    }

    fn render_about_dialog(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        // Calculate popup area (centered)
        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(area)[1];

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(popup_area)[1];

        // Clear the background
        f.render_widget(Clear, popup_area);

        let about_text = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "CCometixLine ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("v", Style::default().fg(Color::Gray)),
                Span::styled(
                    env!("CARGO_PKG_VERSION"),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Features:",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("• 🎨 TUI Configuration Interface"),
            Line::from("• 🎯 Multiple Built-in Themes"),
            Line::from("• ⚡ Real-time Usage Tracking"),
            Line::from("• 💰 Cost Monitoring"),
            Line::from("• 📊 Session Statistics"),
            Line::from("• 🎨 Nerd Font Support"),
            Line::from("• 🔧 Highly Customizable"),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key to continue...",
                Style::default().fg(Color::Yellow),
            )),
        ]);

        let about_dialog = Paragraph::new(about_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("About CCometixLine")
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(about_dialog, popup_area);
    }
}
