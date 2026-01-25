use crate::{App, AppView};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

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
        &[("ESC", "Back to menu"), ("Ctrl+C", "Quit application")],
    ),
];

impl App {
    pub fn draw_help(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Calculate content height: title (1) + blank (1) + sections
        let mut content_height: u16 = 2; // title + blank line
        for (_section_name, items) in HELP_SECTIONS {
            content_height += 1; // section header
            content_height += items.len() as u16; // items
            content_height += 1; // blank line after section
        }
        content_height += 1; // footer

        // Calculate content width (widest line)
        let content_width: u16 = 50;

        // Center the content
        let [centered_area] = Layout::horizontal([Constraint::Length(content_width + 4)])
            .flex(Flex::Center)
            .areas(area);

        let [centered_area] = Layout::vertical([Constraint::Length(content_height + 4)])
            .flex(Flex::Center)
            .areas(centered_area);

        // Draw border
        let block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner_area = block.inner(centered_area);
        frame.render_widget(block, centered_area);

        // Build help content
        let mut lines: Vec<Line> = Vec::new();

        // Title
        lines.push(Line::from(Span::styled(
            "Keyboard Controls",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Sections
        for (section_name, items) in HELP_SECTIONS {
            // Section header
            lines.push(Line::from(Span::styled(
                format!("[{}]", section_name),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));

            // Items
            for (key, description) in *items {
                lines.push(Line::from(vec![
                    Span::styled(format!("<{}>", key), Style::default().fg(Color::Yellow)),
                    Span::styled(format!(" {}", description), Style::default().fg(Color::White)),
                ]));
            }

            lines.push(Line::from(""));
        }

        // Footer
        lines.push(Line::from(Span::styled(
            "Press ESC to return",
            Style::default().fg(Color::DarkGray),
        )));

        frame.render_widget(Paragraph::new(lines).centered(), inner_area);
    }

    pub fn handle_help_input(&mut self, key: KeyEvent) {
        // Any key returns to menu, but ESC is the primary one
        if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Backspace) {
            self.view = AppView::Menu;
        }
    }
}
