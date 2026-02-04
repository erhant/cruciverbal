//! Theme selection view.

use crate::{App, AppView, preferences, theme::Theme};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// State for the theme selection screen.
#[derive(Debug, Default)]
pub struct ThemeSelectState {
    /// Currently hovered theme index.
    pub selected: usize,
}

impl App {
    pub fn draw_theme_select(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Use the hovered theme for live preview
        let preview_theme = Theme::ALL
            .get(self.state.theme_select.selected)
            .unwrap_or(&crate::theme::DEFAULT);

        // Content dimensions
        let content_width: u16 = 30;
        // Title (1) + blank (2) + theme items (5) + blank (2) + footer (1)
        let content_height: u16 = 1 + 2 + Theme::ALL.len() as u16 + 2 + 1;

        // Center the content
        let [centered_area] = Layout::horizontal([Constraint::Length(content_width)])
            .flex(Flex::Center)
            .areas(area);

        let [centered_area] = Layout::vertical([Constraint::Length(content_height)])
            .flex(Flex::Center)
            .areas(centered_area);

        // Build theme selection content
        let mut lines: Vec<Line> = Vec::new();

        // Title - use preview theme colors
        lines.push(Line::from(Span::styled(
            "━━━ Theme ━━━",
            Style::default()
                .fg(preview_theme.secondary)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(""));

        // Theme items
        for (i, theme) in Theme::ALL.iter().enumerate() {
            let is_hovered = i == self.state.theme_select.selected;
            let is_current = theme.id == self.state.theme.id;

            let style = if is_hovered {
                Style::default()
                    .fg(preview_theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(preview_theme.dimmed)
            };

            let prefix = if is_hovered { "▸ " } else { "  " };
            let suffix = if is_current { " ✓" } else { "" };
            lines.push(Line::from(Span::styled(
                format!("{}{}{}", prefix, theme.name, suffix),
                style,
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));

        // Footer - use preview theme colors
        lines.push(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(preview_theme.primary)),
            Span::styled(" navigate · ", Style::default().fg(preview_theme.dimmed)),
            Span::styled("Enter", Style::default().fg(preview_theme.primary)),
            Span::styled(" select · ", Style::default().fg(preview_theme.dimmed)),
            Span::styled("ESC", Style::default().fg(preview_theme.primary)),
            Span::styled(" back", Style::default().fg(preview_theme.dimmed)),
        ]));

        frame.render_widget(Paragraph::new(lines), centered_area);
    }

    pub fn handle_theme_select_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Return to menu without changing theme
                self.view = AppView::Menu;
            }
            KeyCode::Up => {
                if self.state.theme_select.selected > 0 {
                    self.state.theme_select.selected -= 1;
                }
            }
            KeyCode::Down => {
                let theme_count = Theme::ALL.len();
                if self.state.theme_select.selected < theme_count - 1 {
                    self.state.theme_select.selected += 1;
                }
            }
            KeyCode::Enter => {
                // Apply the selected theme
                if let Some(theme) = Theme::ALL.get(self.state.theme_select.selected) {
                    self.state.theme = theme;

                    // Save preference
                    let prefs = preferences::Preferences {
                        theme_id: theme.id.to_string(),
                    };
                    let _ = preferences::save_preferences(&prefs);
                }

                // Return to menu
                self.view = AppView::Menu;
            }
            _ => {}
        }
    }
}
