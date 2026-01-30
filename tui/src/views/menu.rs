use crate::{App, AppView};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
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
        let content_width: u16 = 30;
        // Title (1) + blank (2) + menu items (3) + blank (2) + footer (1)
        let content_height: u16 = 1 + 2 + MenuItem::ALL.len() as u16 + 2 + 1;

        // Center the content
        let [centered_area] = Layout::horizontal([Constraint::Length(content_width)])
            .flex(Flex::Center)
            .areas(area);

        let [centered_area] = Layout::vertical([Constraint::Length(content_height)])
            .flex(Flex::Center)
            .areas(centered_area);

        // Build menu content
        let mut lines: Vec<Line> = Vec::new();

        // Title - larger feel with decoration
        lines.push(Line::from(Span::styled(
            "━━━ Cruciverbal ━━━",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(""));

        // Menu items
        for (i, item) in MenuItem::ALL.iter().enumerate() {
            let style = if i == self.state.menu.sel {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let prefix = if i == self.state.menu.sel { "▸ " } else { "  " };
            lines.push(Line::from(Span::styled(
                format!("{}{}", prefix, item.fmt()),
                style,
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));

        // Footer - more subtle
        lines.push(Line::from(Span::styled(
            "↑↓ navigate · ESC quit",
            Style::default().fg(Color::DarkGray),
        )));

        frame.render_widget(Paragraph::new(lines).centered(), centered_area);
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
