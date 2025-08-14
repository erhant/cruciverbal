use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    Load,
    New,
    Create,
}

pub struct Menu {
    selected_index: usize,
    items: Vec<(String, MenuAction)>,
    input_mode: bool,
    file_path_input: String,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            items: vec![
                ("Load Game".to_string(), MenuAction::Load),
                ("New Random Game".to_string(), MenuAction::New),
                ("Create Puzzle".to_string(), MenuAction::Create),
            ],
            input_mode: false,
            file_path_input: String::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        
        // Clear the background
        frame.render_widget(Clear, area);
        
        if self.input_mode {
            self.render_file_input(frame, area);
        } else {
            self.render_main_menu(frame, area);
        }
    }

    fn render_main_menu(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(5),     // Menu items
                Constraint::Length(3),  // Instructions
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Cruciverbal")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Menu items
        let menu_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, (label, _))| {
                let style = if i == self.selected_index {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(
                    format!("  {}  ", label),
                    style,
                )))
            })
            .collect();

        let menu_list = List::new(menu_items)
            .block(Block::default().borders(Borders::ALL).title(" Menu "));
        frame.render_widget(menu_list, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("↑↓ Navigate • Enter Select • Q Quit")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Controls "));
        frame.render_widget(instructions, chunks[2]);
    }

    fn render_file_input(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(3),  // Input field
                Constraint::Min(3),     // Instructions
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Load Game")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Input field
        let input = Paragraph::new(self.file_path_input.as_str())
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title(" File Path "));
        frame.render_widget(input, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("Enter file path • Enter Load • Esc Cancel")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Instructions "));
        frame.render_widget(instructions, chunks[2]);
    }

    pub fn move_up(&mut self) {
        if !self.input_mode && self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.input_mode && self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn select(&mut self) -> Option<(MenuAction, Option<String>)> {
        if self.input_mode {
            // Return the load action with the file path
            let path = if self.file_path_input.is_empty() {
                None
            } else {
                Some(self.file_path_input.clone())
            };
            self.input_mode = false;
            self.file_path_input.clear();
            Some((MenuAction::Load, path))
        } else {
            let (_, action) = &self.items[self.selected_index];
            match action {
                MenuAction::Load => {
                    self.input_mode = true;
                    None // Don't return action yet, wait for file path
                }
                action => Some((action.clone(), None)),
            }
        }
    }

    pub fn cancel_input(&mut self) {
        if self.input_mode {
            self.input_mode = false;
            self.file_path_input.clear();
        }
    }

    pub fn add_char(&mut self, c: char) {
        if self.input_mode {
            self.file_path_input.push(c);
        }
    }

    pub fn remove_char(&mut self) {
        if self.input_mode {
            self.file_path_input.pop();
        }
    }

    pub fn is_input_mode(&self) -> bool {
        self.input_mode
    }
}