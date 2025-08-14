use crate::{
    editor::Editor,
    events::{AppEvent, EventHandler},
    menu::{Menu, MenuAction},
    ui::UI,
};
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use cruciverbal_core::{Crossword, Direction, generate_crossword, GeneratorConfig};
use cruciverbal_external::GameFile;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Menu,
    Play,
    Edit,
}

pub struct App {
    crossword: Option<Crossword>,
    ui: UI,
    editor: Editor,
    menu: Menu,
    event_handler: EventHandler,
    should_quit: bool,
    mode: AppMode,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            crossword: None,
            ui: UI::new(),
            editor: Editor::new(),
            menu: Menu::new(),
            event_handler: EventHandler::new(Duration::from_millis(250)),
            should_quit: false,
            mode: AppMode::Menu,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the main loop
        let result = self.run_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
        terminal.show_cursor()?;

        result
    }

    fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            terminal.draw(|f| match self.mode {
                AppMode::Menu => self.menu.render(f),
                AppMode::Play => {
                    if let Some(ref crossword) = self.crossword {
                        self.ui.render(f, crossword);
                    }
                }
                AppMode::Edit => {
                    if let Some(ref crossword) = self.crossword {
                        self.editor.render(f, crossword);
                    }
                }
            })?;

            match self.event_handler.next()? {
                AppEvent::Quit => {
                    self.should_quit = true;
                }
                AppEvent::Key(key_event) => {
                    self.handle_key_event(key_event.code)?;
                }
                AppEvent::Tick => {
                    // Handle periodic updates if needed
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_code: KeyCode) -> Result<(), Box<dyn std::error::Error>> {
        // Global commands
        match key_code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return Ok(());
            }
            KeyCode::F(1) => {
                self.mode = AppMode::Play;
                return Ok(());
            }
            KeyCode::F(2) => {
                self.mode = AppMode::Edit;
                return Ok(());
            }
            _ => {}
        }

        // Mode-specific commands
        match self.mode {
            AppMode::Menu => self.handle_menu_mode_key(key_code)?,
            AppMode::Play => self.handle_play_mode_key(key_code)?,
            AppMode::Edit => self.handle_edit_mode_key(key_code)?,
        }

        Ok(())
    }

    fn handle_menu_mode_key(
        &mut self,
        key_code: KeyCode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match key_code {
            KeyCode::Up => {
                self.menu.move_up();
            }
            KeyCode::Down => {
                self.menu.move_down();
            }
            KeyCode::Enter => {
                if let Some((action, file_path)) = self.menu.select() {
                    match action {
                        MenuAction::Load => {
                            if let Some(path) = file_path {
                                self.load_game(&path)?;
                            }
                        }
                        MenuAction::New => {
                            self.create_new_game()?;
                        }
                        MenuAction::Create => {
                            self.create_empty_puzzle();
                            self.mode = AppMode::Edit;
                        }
                    }
                }
            }
            KeyCode::Esc => {
                self.menu.cancel_input();
            }
            KeyCode::Backspace => {
                self.menu.remove_char();
            }
            KeyCode::Char(c) => {
                self.menu.add_char(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_play_mode_key(
        &mut self,
        key_code: KeyCode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.crossword.is_none() {
            return Ok(());
        }

        match key_code {
            KeyCode::Up => {
                self.move_cursor_up();
            }
            KeyCode::Down => {
                self.move_cursor_down();
            }
            KeyCode::Left => {
                self.move_cursor_left();
            }
            KeyCode::Right => {
                self.move_cursor_right();
            }
            KeyCode::Tab => {
                self.ui.toggle_direction();
            }
            KeyCode::Char(c) if c.is_alphabetic() => {
                if let Some(ref mut crossword) = self.crossword {
                    crossword
                        .grid
                        .set_user_input(self.ui.cursor_row, self.ui.cursor_col, c)?;
                    self.move_to_next_cell();
                }
            }
            KeyCode::Backspace => {
                if let Some(ref mut crossword) = self.crossword {
                    crossword
                        .grid
                        .clear_user_input(self.ui.cursor_row, self.ui.cursor_col)?;
                    self.move_to_previous_cell();
                }
            }
            KeyCode::Delete => {
                if let Some(ref mut crossword) = self.crossword {
                    crossword
                        .grid
                        .clear_user_input(self.ui.cursor_row, self.ui.cursor_col)?;
                }
            }
            KeyCode::Enter => {
                if let Some(ref crossword) = self.crossword {
                    self.ui.find_next_empty_cell(crossword);
                }
            }
            KeyCode::Esc => {
                self.mode = AppMode::Menu;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_edit_mode_key(
        &mut self,
        key_code: KeyCode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.crossword.is_none() {
            return Ok(());
        }

        match key_code {
            KeyCode::Up => {
                if let Some(ref crossword) = self.crossword {
                    self.editor.move_cursor(crossword, -1, 0);
                }
            }
            KeyCode::Down => {
                if let Some(ref crossword) = self.crossword {
                    self.editor.move_cursor(crossword, 1, 0);
                }
            }
            KeyCode::Left => {
                if let Some(ref crossword) = self.crossword {
                    self.editor.move_cursor(crossword, 0, -1);
                }
            }
            KeyCode::Right => {
                if let Some(ref crossword) = self.crossword {
                    self.editor.move_cursor(crossword, 0, 1);
                }
            }
            KeyCode::Char(' ') => {
                if let Some(ref mut crossword) = self.crossword {
                    self.editor.toggle_cell(crossword)?;
                }
            }
            KeyCode::Tab => {
                self.editor.toggle_mode();
            }
            KeyCode::Char('c') => {
                // Simple clue creation - in a real editor you'd have a dialog
                if let Some(ref mut crossword) = self.crossword {
                    if self.editor.start_clue_creation() {
                        self.editor.create_clue(
                            crossword,
                            Direction::Across,
                            "Sample clue".to_string(),
                            "ANSWER".to_string(),
                        )?;
                    }
                }
            }
            KeyCode::Esc => {
                self.mode = AppMode::Menu;
            }
            _ => {}
        }
        Ok(())
    }

    fn move_to_next_cell(&mut self) {
        if let Some(ref crossword) = self.crossword {
            if let Some(clue) = crossword.get_clue_at_position(
                self.ui.cursor_row,
                self.ui.cursor_col,
                self.ui.current_direction,
            ) {
                let positions = clue.positions();
                if let Some(current_idx) = positions
                    .iter()
                    .position(|&pos| pos == (self.ui.cursor_row, self.ui.cursor_col))
                {
                    if current_idx + 1 < positions.len() {
                        let (next_row, next_col) = positions[current_idx + 1];
                        self.ui.cursor_row = next_row;
                        self.ui.cursor_col = next_col;
                    }
                }
            }
        }
    }

    fn move_to_previous_cell(&mut self) {
        if let Some(ref crossword) = self.crossword {
            if let Some(clue) = crossword.get_clue_at_position(
                self.ui.cursor_row,
                self.ui.cursor_col,
                self.ui.current_direction,
            ) {
                let positions = clue.positions();
                if let Some(current_idx) = positions
                    .iter()
                    .position(|&pos| pos == (self.ui.cursor_row, self.ui.cursor_col))
                {
                    if current_idx > 0 {
                        let (prev_row, prev_col) = positions[current_idx - 1];
                        self.ui.cursor_row = prev_row;
                        self.ui.cursor_col = prev_col;
                    }
                }
            }
        }
    }

    fn move_cursor_up(&mut self) {
        if let Some(ref crossword) = self.crossword {
            if self.ui.cursor_row > 0 {
                let new_row = self.ui.cursor_row - 1;
                if let Some(cell) = crossword.grid.get_cell(new_row, self.ui.cursor_col) {
                    if cell.is_letter() {
                        self.ui.cursor_row = new_row;
                    }
                }
            }
        }
    }

    fn move_cursor_down(&mut self) {
        if let Some(ref crossword) = self.crossword {
            let new_row = self.ui.cursor_row + 1;
            if new_row < crossword.grid.height {
                if let Some(cell) = crossword.grid.get_cell(new_row, self.ui.cursor_col) {
                    if cell.is_letter() {
                        self.ui.cursor_row = new_row;
                    }
                }
            }
        }
    }

    fn move_cursor_left(&mut self) {
        if let Some(ref crossword) = self.crossword {
            if self.ui.cursor_col > 0 {
                let new_col = self.ui.cursor_col - 1;
                if let Some(cell) = crossword.grid.get_cell(self.ui.cursor_row, new_col) {
                    if cell.is_letter() {
                        self.ui.cursor_col = new_col;
                    }
                }
            }
        }
    }

    fn move_cursor_right(&mut self) {
        if let Some(ref crossword) = self.crossword {
            let new_col = self.ui.cursor_col + 1;
            if new_col < crossword.grid.width {
                if let Some(cell) = crossword.grid.get_cell(self.ui.cursor_row, new_col) {
                    if cell.is_letter() {
                        self.ui.cursor_col = new_col;
                    }
                }
            }
        }
    }

    fn load_game(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let game_file = GameFile::load(path)?;
        self.crossword = Some(game_file.crossword);
        self.mode = AppMode::Play;
        Ok(())
    }

    fn create_new_game(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Sample word list for demonstration
        let sample_words = vec![
            ("DOG".to_string(), "Man's best friend".to_string()),
            ("CAT".to_string(), "Feline pet".to_string()),
            ("RUN".to_string(), "Move quickly".to_string()),
            ("JUMP".to_string(), "Leap into the air".to_string()),
            ("BOOK".to_string(), "Thing you read".to_string()),
            ("TREE".to_string(), "Woody plant".to_string()),
            ("HOUSE".to_string(), "Place to live".to_string()),
            ("WATER".to_string(), "Clear liquid".to_string()),
            ("PHONE".to_string(), "Communication device".to_string()),
            ("COMPUTER".to_string(), "Electronic machine".to_string()),
            ("RAINBOW".to_string(), "Colorful arc in sky".to_string()),
            ("MOUNTAIN".to_string(), "High land formation".to_string()),
        ];

        let config = GeneratorConfig {
            width: 10,
            height: 10,
            max_words: 15,
            min_words: 5,
            symmetry: true,
            max_attempts: 50,
            prefer_longer_words: true,
        };

        match generate_crossword(sample_words, Some(config)) {
            Ok(crossword) => {
                self.crossword = Some(crossword);
                self.mode = AppMode::Play;
                Ok(())
            }
            Err(_e) => {
                // For now, create a simple fallback crossword
                let mut fallback = Crossword::new(
                    "Simple Puzzle".to_string(),
                    "Cruciverbal".to_string(),
                    5,
                    5,
                );
                
                let _ = fallback.add_clue(cruciverbal_core::Clue::new(
                    1,
                    Direction::Across,
                    "Man's best friend".to_string(),
                    "DOG".to_string(),
                    0,
                    0,
                ));
                
                self.crossword = Some(fallback);
                self.mode = AppMode::Play;
                Ok(())
            }
        }
    }

    fn create_empty_puzzle(&mut self) {
        let crossword = Crossword::new(
            "New Puzzle".to_string(),
            "Author".to_string(),
            15,
            15,
        );
        self.crossword = Some(crossword);
    }
}
