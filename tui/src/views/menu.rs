use crate::{App, AppView};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::text::Span;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{List, ListItem, Paragraph},
};

#[derive(Default, Debug)]
pub struct MenuState {
    /// Selected menu item index.
    pub sel: usize,
}

/// A menu item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuItem {
    Play,
    Exit,
}

impl MenuItem {
    pub const ALL: [MenuItem; 2] = [MenuItem::Play, MenuItem::Exit];
    pub fn fmt(&self) -> String {
        match self {
            // TODO: may add `New Game`, `Load Game`, `Settings`, etc.
            MenuItem::Play => "Play".to_string(),
            MenuItem::Exit => "Exit".to_string(),
        }
    }
}

impl App {
    pub fn draw_menu(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // create layout
        let vertical = Layout::vertical([
            Constraint::Length(2), // Header
            Constraint::Min(0),    // Menu
            Constraint::Length(2), // Footer
        ]);
        let [header_area, menu_area, footer_area] = vertical.areas(area);

        let [_, menu_area, _] =
            Layout::horizontal(Constraint::from_percentages([40, 20, 40])).areas(menu_area);

        // header
        frame.render_widget(Paragraph::new("Cruciverbal").centered(), header_area);

        // menu items
        let menu_items: Vec<ListItem> = MenuItem::ALL
            .iter()
            .enumerate()
            .map(|(i, item)| {
                ListItem::new(item.fmt()).style(if i == self.state.menu.sel {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK)
                } else {
                    Style::default()
                })
            })
            .collect();
        frame.render_widget(List::new(menu_items), menu_area);

        // footer
        let footer_line = Line::from_iter([Span::styled(
            "↑↓ navigate • ESC quit",
            Style::default().fg(Color::DarkGray),
        )]);
        frame.render_widget(
            Paragraph::new(footer_line)
                .style(Style::default().fg(Color::DarkGray))
                .centered(),
            footer_area,
        );
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
            MenuItem::Play => {
                use crate::views::game::GameView;

                self.view = match self.state.game.puzzle {
                    Some(_) => AppView::Game(GameView::Playing),
                    None => AppView::Game(GameView::Selecting),
                };
            }
            MenuItem::Exit => {
                self.quit();
            }
        }
    }
}
