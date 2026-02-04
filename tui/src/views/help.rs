use crate::{App, AppView};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::time::Instant;

/// Help content sections with their keyboard shortcuts.
const HELP_SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "Navigation",
        &[
            ("Arrow keys", "Move between cells"),
            ("Shift + Arrow", "Jump to next word"),
            ("Space", "Toggle direction (Across/Down)"),
        ],
    ),
    (
        "Input",
        &[("A-Z", "Enter letter"), ("Backspace/Delete", "Clear cell")],
    ),
    (
        "Reveal",
        &[
            ("Ctrl+R", "Reveal current letter"),
            ("Shift+Ctrl+R", "Reveal current word"),
            ("Alt+Ctrl+R", "Reveal entire puzzle"),
        ],
    ),
    (
        "General",
        &[
            ("Ctrl+S", "Save game"),
            ("Ctrl+H", "Show help"),
            ("ESC", "Back to menu"),
            ("Ctrl+C", "Quit application"),
        ],
    ),
];

impl App {
    pub fn draw_help(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let theme = self.state.theme;

        // Calculate content height: title (1) + blank (1) + sections
        let mut content_height: u16 = 2; // title + blank line
        for (_section_name, items) in HELP_SECTIONS {
            content_height += 1; // section header
            content_height += items.len() as u16; // items
            content_height += 1; // blank line after section
        }
        content_height += 1; // footer

        // Calculate content width (widest line)
        let content_width: u16 = 40;

        // Center the content
        let [centered_area] = Layout::horizontal([Constraint::Length(content_width)])
            .flex(Flex::Center)
            .areas(area);

        let [centered_area] = Layout::vertical([Constraint::Length(content_height)])
            .flex(Flex::Center)
            .areas(centered_area);

        // Build help content
        let mut lines: Vec<Line> = Vec::new();

        // Title
        lines.push(Line::from(Span::styled(
            "━━━ Keyboard Controls ━━━",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Sections
        for (section_name, items) in HELP_SECTIONS {
            // Section header - more subtle
            lines.push(Line::from(Span::styled(
                section_name.to_string(),
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )));

            // Items
            for (key, description) in *items {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {}", key), Style::default().fg(theme.secondary)),
                    Span::styled(
                        format!("  {}", description),
                        Style::default().fg(theme.dimmed),
                    ),
                ]));
            }

            lines.push(Line::from(""));
        }

        // Footer
        lines.push(Line::from(vec![
            Span::styled("ESC", Style::default().fg(theme.primary)),
            Span::styled(" to return", Style::default().fg(theme.dimmed)),
        ]));

        frame.render_widget(Paragraph::new(lines), centered_area);
    }

    pub fn handle_help_input(&mut self, key: KeyEvent) {
        // Any key returns, but ESC is the primary one
        if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Backspace) {
            // Return to previous view if set, otherwise go to menu
            if let Some(prev) = self.previous_view.take() {
                // If returning to game, resume timer
                if matches!(prev, AppView::Game(_)) {
                    if let Some(elapsed) = self.state.game.paused_elapsed.take() {
                        self.state.game.start_time = Some(Instant::now() - elapsed);
                    }
                }
                self.view = prev;
            } else {
                self.view = AppView::Menu;
            }
        }
    }
}
