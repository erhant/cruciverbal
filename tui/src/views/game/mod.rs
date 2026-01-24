use crate::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use cruciverbal_providers::PuzzleProvider;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::time::Instant;

mod constants;

mod grid;
use grid::*;

mod cell;
pub use cell::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum GameView {
    /// User is playing the puzzle, loaded within [`GameState::puzzle`].
    Playing,
    /// User is selecting a puzzle to load.
    #[default]
    Selecting,
    /// Puzzle is being loaded (either from file or network).
    Loading,
    /// User is saving the current puzzle to file.
    Saving,
}

/// Which field is currently active in the selection screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionField {
    #[default]
    Date,
    Provider,
    Start,
}

impl SelectionField {
    /// Move to the next field.
    pub fn next(&self) -> Self {
        match self {
            SelectionField::Date => SelectionField::Provider,
            SelectionField::Provider => SelectionField::Start,
            SelectionField::Start => SelectionField::Date,
        }
    }

    /// Move to the previous field.
    pub fn prev(&self) -> Self {
        match self {
            SelectionField::Date => SelectionField::Start,
            SelectionField::Provider => SelectionField::Date,
            SelectionField::Start => SelectionField::Provider,
        }
    }
}

/// State for the puzzle selection screen.
#[derive(Debug, Clone)]
pub struct SelectionState {
    /// Date input string (YYYY-MM-DD format).
    pub date: String,
    /// Currently selected provider index.
    pub provider_idx: usize,
    /// Which field is currently active.
    pub active_field: SelectionField,
    /// Error message to display, if any.
    pub error: Option<String>,
}

impl Default for SelectionState {
    fn default() -> Self {
        // Default to today's date
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        Self {
            date: today,
            provider_idx: 0,
            active_field: SelectionField::Date,
            error: None,
        }
    }
}

#[derive(Debug)]
pub struct GameState {
    /// Loaded puzzle, if any.
    pub puzzle: Option<puz_parse::Puzzle>,

    /// The playable grid built from the puzzle.
    pub grid: Option<PuzzleGrid>,

    /// Selected cell (row, col).
    pub sel: (usize, usize),

    /// Active direction for navigation and clue display.
    /// Toggled with SPACEBAR.
    pub active_direction: Direction,

    /// Visible area dimensions (width, height) in terminal cells.
    pub visible_area: (u16, u16),

    /// Puzzle date string (e.g., "2025-12-08").
    pub puzzle_date: Option<String>,

    /// Time when the puzzle was started (for timer display).
    pub start_time: Option<Instant>,

    /// State for the puzzle selection screen.
    pub selection: SelectionState,

    /* scrollbar stuff */
    /// Current scroll position (vertical, horizontal).
    pub scroll_cur: (u16, u16),
    /// Maximum scroll position (vertical, horizontal), be careful about this as it may crash the app
    /// if set incorrectly.
    pub scroll_max: (u16, u16),
    /// Scrollbar state for the [`Scrollbar`] widget (vertical, horizontal).
    pub scroll_bar: (ScrollbarState, ScrollbarState),
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            puzzle: None,
            grid: None,
            sel: (0, 0),
            active_direction: Direction::Across,
            visible_area: (0, 0),
            puzzle_date: None,
            start_time: None,
            selection: SelectionState::default(),
            scroll_cur: (0, 0),
            scroll_max: (0, 0),
            scroll_bar: (ScrollbarState::default(), ScrollbarState::default()),
        }
    }
}

impl GameState {
    /// Reset the game state for a new game, keeping selection state fresh.
    pub fn reset_for_new_game(&mut self) {
        // TODO: use default?
        self.puzzle = None;
        self.grid = None;
        self.sel = (0, 0);
        self.active_direction = Direction::Across;
        self.visible_area = (0, 0);
        self.puzzle_date = None;
        self.start_time = None;
        self.selection = SelectionState::default();
        self.scroll_cur = (0, 0);
        self.scroll_max = (0, 0);
        self.scroll_bar = (ScrollbarState::default(), ScrollbarState::default());
    }
}

impl App {
    pub fn draw_game(&mut self, view: GameView, frame: &mut ratatui::Frame) {
        match view {
            GameView::Playing => self.draw_game_playing(frame),
            GameView::Selecting => self.draw_game_selecting(frame),
            GameView::Loading => self.draw_game_loading(frame),
            GameView::Saving => todo!(),
        }
    }

    fn draw_game_selecting(&mut self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        // Create centered layout
        let vertical = Layout::vertical([
            Constraint::Min(1),     // Top padding
            Constraint::Length(12), // Content area
            Constraint::Min(1),     // Bottom padding
        ]);
        let [_, content_area, _] = vertical.areas(area);

        let horizontal = Layout::horizontal([
            Constraint::Min(1),     // Left padding
            Constraint::Length(40), // Form area
            Constraint::Min(1),     // Right padding
        ]);
        let [_, form_area, _] = horizontal.areas(content_area);

        // draw the selection form
        let block = Block::default()
            .title(" New Game ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner_area = block.inner(form_area);
        frame.render_widget(block, form_area);

        // split inner area into rows
        let rows = Layout::vertical([
            Constraint::Length(3), // Date field
            Constraint::Length(3), // Provider field
            Constraint::Length(2), // Start button
            Constraint::Length(2), // Error message
        ])
        .split(inner_area);

        let selection = &self.state.game.selection;

        // render date field
        let date_style = if selection.active_field == SelectionField::Date {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let date_block = Block::default()
            .title(" Date (YYYY-MM-DD) ")
            .borders(Borders::ALL)
            .border_style(date_style);
        let date_inner = date_block.inner(rows[0]);
        frame.render_widget(date_block, rows[0]);

        let cursor_char = if selection.active_field == SelectionField::Date {
            "_"
        } else {
            ""
        };
        let date_text = format!("{}{}", selection.date, cursor_char);
        frame.render_widget(Paragraph::new(date_text).style(date_style), date_inner);

        // render provider field
        let provider_style = if selection.active_field == SelectionField::Provider {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let provider_block = Block::default()
            .title(" Provider ")
            .borders(Borders::ALL)
            .border_style(provider_style);
        let provider_inner = provider_block.inner(rows[1]);
        frame.render_widget(provider_block, rows[1]);

        let provider_name = PuzzleProvider::ALL
            .get(selection.provider_idx)
            .map(|p| p.name())
            .unwrap_or("Unknown");
        let provider_text = format!("< {} >", provider_name);
        frame.render_widget(
            Paragraph::new(provider_text)
                .style(provider_style)
                .centered(),
            provider_inner,
        );

        // render start button
        let start_style = if selection.active_field == SelectionField::Start {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };
        frame.render_widget(
            Paragraph::new("[ Start Game ]")
                .style(start_style)
                .centered(),
            rows[2],
        );

        // show error message if any
        if let Some(ref error) = selection.error {
            frame.render_widget(
                Paragraph::new(error.as_str())
                    .style(Style::default().fg(Color::Red))
                    .centered(),
                rows[3],
            );
        }

        // footer with instructions
        let footer_area =
            Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area)[1];
        let footer = Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::styled(" navigate • ", Style::default().fg(Color::DarkGray)),
            Span::styled("←→", Style::default().fg(Color::Yellow)),
            Span::styled(" change • ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::styled(" confirm • ", Style::default().fg(Color::DarkGray)),
            Span::styled("ESC", Style::default().fg(Color::Yellow)),
            Span::styled(" back", Style::default().fg(Color::DarkGray)),
        ]);
        frame.render_widget(Paragraph::new(footer).centered(), footer_area);
    }

    fn draw_game_loading(&mut self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        let vertical = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [_, content_area, _] = vertical.areas(area);

        let loading_text = format!("Loading puzzle for {}...", self.state.game.selection.date);
        frame.render_widget(
            Paragraph::new(loading_text)
                .style(Style::default().fg(Color::Cyan))
                .centered(),
            content_area,
        );
    }

    fn draw_game_playing(&mut self, frame: &mut ratatui::Frame) {
        // initialize grid from puzzle if not already done
        if self.state.game.grid.is_none() {
            if let Some(puzzle) = self.state.game.puzzle.as_ref() {
                let mut grid = PuzzleGrid::from_solution(&puzzle.grid.solution);

                // find and select the first letter cell
                if let Some((row, col)) = grid.find_first_letter_cell() {
                    grid.set_selection(row, col, self.state.game.active_direction);
                    self.state.game.sel = (row, col);
                }

                self.state.game.grid = Some(grid);

                // start timer when grid is initialized
                if self.state.game.start_time.is_none() {
                    self.state.game.start_time = Some(Instant::now());
                }
            }
        }

        let Some(grid) = self.state.game.grid.as_ref() else {
            return; // nothing to draw
        };

        // Calculate content dimensions
        // Grid: each cell is 4x4, plus 1 for final border
        let grid_content_width = (grid.width() as u16 * 4) + 1;
        let grid_content_height = (grid.height() as u16 * 4) + 1;

        // Total content: header (3) + grid + footer (3)
        // Header/footer have 1 line content + 2 lines border (top+bottom shared with grid)
        let total_height = 3 + grid_content_height + 3;
        let total_width = grid_content_width.max(40); // minimum width for header/footer text

        let full_area = frame.area();

        // Center horizontally
        let [centered_area] = Layout::horizontal([Constraint::Length(total_width)])
            .flex(Flex::Center)
            .areas(full_area);

        // Center vertically
        let [centered_area] = Layout::vertical([Constraint::Length(total_height)])
            .flex(Flex::Center)
            .areas(centered_area);

        // Split into 3 areas: top bar, grid, bottom bar
        let layout = Layout::vertical([
            Constraint::Length(3), // top bar with borders
            Constraint::Min(1),    // grid area
            Constraint::Length(3), // bottom bar with borders
        ])
        .split(centered_area);

        let top_area = layout[0];
        let grid_area = layout[1];
        let bottom_area = layout[2];

        // === TOP BAR ===
        self.draw_top_bar(frame, top_area);

        // === GRID ===
        self.state.game.visible_area = (grid_area.width, grid_area.height);

        let mut par = grid.to_par();
        let (width, height) = (grid_area.width, grid_area.height);

        // calculate vertical scroll bounds
        let content_height = (grid.height() as u16 * 4) + 1;
        let max_scroll_v = content_height.saturating_sub(height.saturating_sub(1));

        // calculate horizontal scroll bounds
        let content_width = (grid.width() as u16 * 4) + 1;
        let max_scroll_h = content_width.saturating_sub(width.saturating_sub(1));

        self.state.game.scroll_max = (max_scroll_v, max_scroll_h);

        // clamp scroll position
        self.state.game.scroll_cur.0 = self
            .state
            .game
            .scroll_cur
            .0
            .min(self.state.game.scroll_max.0);
        self.state.game.scroll_cur.1 = self
            .state
            .game
            .scroll_cur
            .1
            .min(self.state.game.scroll_max.1);

        par = par.scroll((self.state.game.scroll_cur.0, self.state.game.scroll_cur.1));
        frame.render_widget(par, grid_area);

        // render vertical scrollbar
        self.state.game.scroll_bar.0 = self
            .state
            .game
            .scroll_bar
            .0
            .content_length(self.state.game.scroll_max.0 as usize)
            .position(self.state.game.scroll_cur.0 as usize);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            grid_area,
            &mut self.state.game.scroll_bar.0,
        );

        // render horizontal scrollbar
        self.state.game.scroll_bar.1 = self
            .state
            .game
            .scroll_bar
            .1
            .content_length(self.state.game.scroll_max.1 as usize)
            .position(self.state.game.scroll_cur.1 as usize);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                .begin_symbol(Some("←"))
                .end_symbol(Some("→")),
            grid_area,
            &mut self.state.game.scroll_bar.1,
        );

        // === BOTTOM BAR (CLUE) ===
        self.draw_clue_bar(frame, bottom_area);
    }

    /// Draw the top bar: date (left), title (center), timer (right).
    fn draw_top_bar(&self, frame: &mut ratatui::Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let date_str = self.state.game.puzzle_date.as_deref().unwrap_or("No date");

        let title_str = self
            .state
            .game
            .puzzle
            .as_ref()
            .map(|p| p.info.title.as_str())
            .unwrap_or("Untitled");

        let timer_str = match self.state.game.start_time {
            Some(start) => {
                let elapsed = start.elapsed();
                let mins = elapsed.as_secs() / 60;
                let secs = elapsed.as_secs() % 60;
                format!("{:02}:{:02}", mins, secs)
            }
            None => "00:00".to_string(),
        };

        // Calculate spacing for centering the title
        let total_width = inner.width as usize;
        let date_len = date_str.len();
        let timer_len = timer_str.len();
        let title_len = title_str.len();

        // Try to center the title
        let left_space = (total_width.saturating_sub(title_len)) / 2;
        let right_start = left_space + title_len;

        let dim_style = Style::default().fg(Color::DarkGray);
        let title_style = Style::default().fg(Color::Cyan);
        let timer_style = Style::default().fg(Color::Yellow);

        // Build the line with proper spacing
        let mut spans = vec![Span::styled(date_str, dim_style)];

        // Padding between date and title
        let pad_left = left_space.saturating_sub(date_len);
        if pad_left > 0 {
            spans.push(Span::raw(" ".repeat(pad_left)));
        }

        spans.push(Span::styled(title_str, title_style));

        // Padding between title and timer
        let pad_right = total_width
            .saturating_sub(right_start)
            .saturating_sub(timer_len);
        if pad_right > 0 {
            spans.push(Span::raw(" ".repeat(pad_right)));
        }

        spans.push(Span::styled(timer_str, timer_style));

        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), inner);
    }

    /// Draw the clue bar based on currently selected cell.
    fn draw_clue_bar(&self, frame: &mut ratatui::Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let clue_text = self.get_current_clue();
        let clue_style = Style::default().fg(Color::White);
        let par = Paragraph::new(Span::styled(clue_text, clue_style));
        frame.render_widget(par, inner);
    }

    /// Get the clue text for the currently selected cell based on active direction.
    fn get_current_clue(&self) -> String {
        let Some(grid) = self.state.game.grid.as_ref() else {
            return String::new();
        };

        let Some(puzzle) = self.state.game.puzzle.as_ref() else {
            return String::new();
        };

        let (row, col) = self.state.game.sel;
        let Some(cell) = grid.get(row, col) else {
            return String::new();
        };

        let direction = self.state.game.active_direction;

        // Get clue number for the active direction
        let clue_no = cell.clue_no_for_direction(direction);

        // If cell doesn't have a clue in the active direction, try the other direction
        let (clue_no, dir_char) = match clue_no {
            Some(n) => (
                n,
                if direction == Direction::Across {
                    'A'
                } else {
                    'D'
                },
            ),
            None => {
                // Fall back to the other direction
                let other_dir = direction.toggle();
                match cell.clue_no_for_direction(other_dir) {
                    Some(n) => (
                        n,
                        if other_dir == Direction::Across {
                            'A'
                        } else {
                            'D'
                        },
                    ),
                    None => return String::new(),
                }
            }
        };

        let clue_text = if dir_char == 'A' {
            puzzle
                .clues
                .across
                .get(&(clue_no as u16))
                .map(|s| s.as_str())
                .unwrap_or("?")
        } else {
            puzzle
                .clues
                .down
                .get(&(clue_no as u16))
                .map(|s| s.as_str())
                .unwrap_or("?")
        };

        format!("{}{}: {}", clue_no, dir_char, clue_text)
    }

    pub fn handle_game_input(&mut self, view: GameView, key: KeyEvent) {
        match view {
            GameView::Selecting => self.handle_selecting_input(key),
            GameView::Loading => {
                // ESC cancels loading and goes back to menu
                if key.code == KeyCode::Esc {
                    use crate::AppView;
                    self.view = AppView::Menu;
                }
            }
            GameView::Playing => self.handle_playing_input(key),
            GameView::Saving => {}
        }
    }

    fn handle_selecting_input(&mut self, key: KeyEvent) {
        use crate::AppView;

        match key.code {
            KeyCode::Esc => {
                self.view = AppView::Menu;
            }

            KeyCode::Up => {
                self.state.game.selection.active_field =
                    self.state.game.selection.active_field.prev();
            }

            KeyCode::Down | KeyCode::Tab => {
                self.state.game.selection.active_field =
                    self.state.game.selection.active_field.next();
            }

            KeyCode::Left => {
                if self.state.game.selection.active_field == SelectionField::Provider {
                    let len = PuzzleProvider::ALL.len();
                    if self.state.game.selection.provider_idx == 0 {
                        self.state.game.selection.provider_idx = len - 1;
                    } else {
                        self.state.game.selection.provider_idx -= 1;
                    }
                }
            }

            KeyCode::Right => {
                if self.state.game.selection.active_field == SelectionField::Provider {
                    let len = PuzzleProvider::ALL.len();
                    self.state.game.selection.provider_idx =
                        (self.state.game.selection.provider_idx + 1) % len;
                }
            }

            KeyCode::Enter => {
                if self.state.game.selection.active_field == SelectionField::Start {
                    // Validate date format before starting
                    if self.validate_date() {
                        self.state.game.selection.error = None;
                        self.view = AppView::Game(GameView::Loading);
                    }
                } else {
                    // Move to next field
                    self.state.game.selection.active_field =
                        self.state.game.selection.active_field.next();
                }
            }

            KeyCode::Char(c) => {
                if self.state.game.selection.active_field == SelectionField::Date {
                    // Only allow digits and dashes for date input
                    if c.is_ascii_digit() || c == '-' {
                        if self.state.game.selection.date.len() < 10 {
                            self.state.game.selection.date.push(c);
                            self.state.game.selection.error = None;
                        }
                    }
                }
            }

            KeyCode::Backspace => {
                if self.state.game.selection.active_field == SelectionField::Date {
                    self.state.game.selection.date.pop();
                    self.state.game.selection.error = None;
                }
            }

            _ => {}
        }
    }

    fn validate_date(&mut self) -> bool {
        if chrono::NaiveDate::parse_from_str(&self.state.game.selection.date, "%Y-%m-%d").is_err() {
            self.state.game.selection.error = Some("Invalid date format".to_string());
            false
        } else {
            true
        }
    }

    fn handle_playing_input(&mut self, key: KeyEvent) {
        if self.state.game.grid.is_none() {
            return;
        }

        // special reveal commands
        // Note: CTRL+R may be reported as '\x12' (control character for R) on some terminals
        let is_ctrl_r = matches!(key.code, KeyCode::Char('r') | KeyCode::Char('R'))
            && key.modifiers.contains(KeyModifiers::CONTROL);
        let is_ctrl_r_char = key.code == KeyCode::Char('\x12');

        if is_ctrl_r {
            // SHIFT+CTRL+R: reveal current word
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                self.reveal_current_word();
                self.advance_to_next_cell();
                return;
            }
            // ALT+CTRL+R: reveal entire puzzle
            else if key.modifiers.contains(KeyModifiers::ALT) {
                if let Some(grid) = self.state.game.grid.as_mut() {
                    grid.reveal_all();
                }
                return;
            }
            // CTRL+R: reveal current letter
            else {
                self.reveal_current_letter();
                self.advance_to_next_cell();
                return;
            }
        } else if is_ctrl_r_char {
            // CTRL+R as control character: reveal current letter
            self.reveal_current_letter();
            self.advance_to_next_cell();
            return;
        }

        match key.code {
            // ESC: go back to menu
            KeyCode::Esc => {
                use crate::AppView;
                self.view = AppView::Menu;
            }

            // SPACEBAR: toggle direction (Across <-> Down)
            KeyCode::Char(' ') => {
                self.toggle_direction();
            }

            // navigation: move selection with arrow keys
            KeyCode::Up => self.move_selection(-1, 0),
            KeyCode::Down => self.move_selection(1, 0),
            KeyCode::Left => self.move_selection(0, -1),
            KeyCode::Right => self.move_selection(0, 1),

            // letter input: A-Z (and lowercase a-z)
            KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                let letter = c.to_ascii_uppercase();
                let (row, col) = self.state.game.sel;
                if let Some(grid) = self.state.game.grid.as_mut() {
                    if let Some(cell) = grid.get_mut(row, col) {
                        cell.set_user_letter(Some(letter));
                    }
                }
                // auto-advance to next cell in active direction
                self.advance_to_next_cell();
            }

            // backspace/delete: clear the current cell
            KeyCode::Backspace | KeyCode::Delete => {
                let (row, col) = self.state.game.sel;
                if let Some(grid) = self.state.game.grid.as_mut() {
                    if let Some(cell) = grid.get_mut(row, col) {
                        cell.set_user_letter(None);
                    }
                }
            }

            _ => {}
        }
    }

    /// Toggle the active direction between Across and Down.
    fn toggle_direction(&mut self) {
        self.state.game.active_direction = self.state.game.active_direction.toggle();

        // re-apply selection to update word highlighting
        let (row, col) = self.state.game.sel;
        let direction = self.state.game.active_direction;
        if let Some(grid) = self.state.game.grid.as_mut() {
            grid.set_selection(row, col, direction);
        }
    }

    /// Reveal the current cell's letter.
    fn reveal_current_letter(&mut self) {
        let (row, col) = self.state.game.sel;
        if let Some(grid) = self.state.game.grid.as_mut() {
            if let Some(cell) = grid.get_mut(row, col) {
                cell.reveal();
            }
        }
    }

    /// Reveal all letters in the currently selected word.
    fn reveal_current_word(&mut self) {
        let (row, col) = self.state.game.sel;
        let direction = self.state.game.active_direction;

        // Get the clue number for the current cell in the active direction
        let clue_no = if let Some(grid) = self.state.game.grid.as_ref() {
            if let Some(cell) = grid.get(row, col) {
                cell.clue_no_for_direction(direction)
            } else {
                None
            }
        } else {
            None
        };

        // Reveal all cells in the word
        if let Some(clue_no) = clue_no {
            if let Some(grid) = self.state.game.grid.as_mut() {
                grid.reveal_word(clue_no, direction);
            }
        }
    }

    /// Move selection by the given delta, skipping filled cells.
    fn move_selection(&mut self, row_delta: i32, col_delta: i32) {
        let Some(grid) = self.state.game.grid.as_mut() else {
            return;
        };

        let direction = self.state.game.active_direction;
        let (cur_row, cur_col) = self.state.game.sel;
        let height = grid.height() as i32;
        let width = grid.width() as i32;

        let mut new_row = cur_row as i32 + row_delta;
        let mut new_col = cur_col as i32 + col_delta;

        // keep moving in the same direction until we find a non-filled cell or hit boundary
        while new_row >= 0 && new_row < height && new_col >= 0 && new_col < width {
            if let Some(cell) = grid.get(new_row as usize, new_col as usize) {
                if !cell.is_filled() {
                    // Found a valid cell, update selection
                    if grid.set_selection(new_row as usize, new_col as usize, direction) {
                        self.state.game.sel = (new_row as usize, new_col as usize);
                    }
                    break;
                }
            }
            // skip filled cells, continue in same direction
            new_row += row_delta;
            new_col += col_delta;
        }

        // now update scroll position (grid borrow is dropped)
        self.ensure_selection_visible();
    }

    /// Advance to the next empty cell in the active direction after entering a letter.
    ///
    /// - If direction is `Across`: move right, wrap to next row if at end
    /// - If direction is `Down`: move down, wrap to next column if at end
    fn advance_to_next_cell(&mut self) {
        let Some(grid) = self.state.game.grid.as_mut() else {
            return;
        };

        let direction = self.state.game.active_direction;
        let (cur_row, cur_col) = self.state.game.sel;
        let height = grid.height() as usize;
        let width = grid.width() as usize;

        // determine the delta based on direction
        let (row_delta, col_delta): (i32, i32) = match direction {
            Direction::Across => (0, 1),
            Direction::Down => (1, 0),
        };

        let mut new_row = cur_row as i32 + row_delta;
        let mut new_col = cur_col as i32 + col_delta;

        // try to find the next non-filled cell
        loop {
            // check if we've gone past the grid boundary
            if new_row < 0 || new_row >= height as i32 || new_col < 0 || new_col >= width as i32 {
                // wrap to next row/column
                match direction {
                    Direction::Across => {
                        new_row += 1;
                        new_col = 0;
                    }
                    Direction::Down => {
                        new_row = 0;
                        new_col += 1;
                    }
                }

                // if we've wrapped past the entire grid, stop
                if new_row >= height as i32 || new_col >= width as i32 {
                    break;
                }
            }

            // check if this cell is valid (not filled)
            if let Some(cell) = grid.get(new_row as usize, new_col as usize) {
                if !cell.is_filled() {
                    // found a valid cell, update selection
                    if grid.set_selection(new_row as usize, new_col as usize, direction) {
                        self.state.game.sel = (new_row as usize, new_col as usize);
                    }
                    break;
                }
            }

            // move to next cell
            new_row += row_delta;
            new_col += col_delta;
        }

        // Update scroll position
        self.ensure_selection_visible();
    }

    /// Adjust scroll position to ensure the selected cell is visible.
    fn ensure_selection_visible(&mut self) {
        let (sel_row, sel_col) = self.state.game.sel;
        let (visible_w, visible_h) = self.state.game.visible_area;

        // Cell rendering dimensions:
        // - Each cell is 4 lines tall (top border shared, content)
        // - Each cell is 4 chars wide (left border shared, content)
        // - Plus 1 for bottom/right border on last row/col

        // Calculate the pixel position of the selected cell
        let cell_top = (sel_row as u16) * 4; // top of cell in content coordinates
        let cell_bottom = cell_top + 4; // bottom of cell
        let cell_left = (sel_col as u16) * 4; // left of cell
        let cell_right = cell_left + 4; // right of cell

        let (scroll_v, scroll_h) = self.state.game.scroll_cur;

        // vertical scrolling
        let visible_top = scroll_v;
        let visible_bottom = scroll_v + visible_h.saturating_sub(2); // account for scrollbar
        let new_scroll_v = if cell_top < visible_top {
            // cell is above visible area
            cell_top
        } else if cell_bottom > visible_bottom {
            // cell is below visible area
            cell_bottom.saturating_sub(visible_h.saturating_sub(2)) // account for scrollbar
        } else {
            scroll_v
        };

        // horizontal scrolling
        let visible_left = scroll_h;
        let visible_right = scroll_h + visible_w.saturating_sub(2); // account for scrollbar
        let new_scroll_h = if cell_left < visible_left {
            // cell is to the left of visible area
            cell_left
        } else if cell_right > visible_right {
            // cell is to the right of visible area
            cell_right.saturating_sub(visible_w.saturating_sub(2)) // account for scrollbar
        } else {
            scroll_h
        };

        self.state.game.scroll_cur = (
            new_scroll_v.min(self.state.game.scroll_max.0),
            new_scroll_h.min(self.state.game.scroll_max.1),
        );
    }
}
