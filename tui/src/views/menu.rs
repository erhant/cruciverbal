use crate::{App, AppView};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

#[derive(Default, Debug)]
pub struct MenuState {
    /// Selected menu item index.
    pub sel: usize,
}

/// A menu item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuItem {
    NewGame,
    Help,
    Exit,
}

impl MenuItem {
    pub const ALL: [MenuItem; 3] = [MenuItem::NewGame, MenuItem::Help, MenuItem::Exit];
    pub fn fmt(&self) -> String {
        match self {
            MenuItem::NewGame => "New Game".to_string(),
            MenuItem::Help => "Help".to_string(),
            MenuItem::Exit => "Exit".to_string(),
        }
    }
}

impl App {
    pub fn draw_menu(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Content dimensions
        let content_width: u16 = 24;
        // Title (1) + blank (1) + menu items (3) + blank (1) + footer (1)
        let content_height: u16 = 1 + 1 + MenuItem::ALL.len() as u16 + 1 + 1;

        // Center the content
        let [centered_area] = Layout::horizontal([Constraint::Length(content_width + 4)])
            .flex(Flex::Center)
            .areas(area);

        let [centered_area] = Layout::vertical([Constraint::Length(content_height + 4)])
            .flex(Flex::Center)
            .areas(centered_area);

        // Draw border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner_area = block.inner(centered_area);
        frame.render_widget(block, centered_area);

        // Build menu content
        let mut lines: Vec<Line> = Vec::new();

        // Title
        lines.push(Line::from(Span::styled(
            "Cruciverbal",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Menu items
        for (i, item) in MenuItem::ALL.iter().enumerate() {
            let style = if i == self.state.menu.sel {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if i == self.state.menu.sel { "> " } else { "  " };
            lines.push(Line::from(Span::styled(
                format!("{}{}", prefix, item.fmt()),
                style,
            )));
        }

        lines.push(Line::from(""));

        // Footer
        lines.push(Line::from(vec![
            Span::styled("<", Style::default().fg(Color::Yellow)),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::styled("> navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("<", Style::default().fg(Color::Yellow)),
            Span::styled("ESC", Style::default().fg(Color::Yellow)),
            Span::styled("> quit", Style::default().fg(Color::DarkGray)),
        ]));

        frame.render_widget(Paragraph::new(lines).centered(), inner_area);
    }

    pub fn handle_menu_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.quit(),
            KeyCode::Up => self.menu_up(),
            KeyCode::Down => self.menu_down(),
            KeyCode::Enter => self.select_menu_item(),
            _ => {}
        }
    }

    fn menu_up(&mut self) {
        if self.state.menu.sel > 0 {
            self.state.menu.sel -= 1;
        }
    }

    fn menu_down(&mut self) {
        let menu_count = MenuItem::ALL.len();
        if self.state.menu.sel < menu_count - 1 {
            self.state.menu.sel += 1;
        }
    }

    fn select_menu_item(&mut self) {
        // TODO: can use `.get` here for safety
        match MenuItem::ALL[self.state.menu.sel] {
            MenuItem::NewGame => {
                use crate::views::game::GameView;

                // Reset game state for a new game
                self.state.game.reset_for_new_game();
                self.view = AppView::Game(GameView::Selecting);
            }
            MenuItem::Help => {
                self.view = AppView::Help;
            }
            MenuItem::Exit => {
                self.quit();
            }
        }
    }
}
