use crate::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use cruciverbal_providers::PuzzleProvider;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::time::{Duration, Instant};

/// Format a duration as MM:SS, defaulting to "00:00" if None.
fn format_duration(duration: Option<Duration>) -> String {
    match duration {
        Some(d) => format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60),
        None => "00:00".to_string(),
    }
}

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
    /// Puzzle is completed correctly, showing congratulations popup.
    Completed,
    /// User continues playing after completion (timer stopped, no validation).
    CompletedPlaying,
}

/// Completion state for the puzzle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompletionState {
    /// Puzzle is not fully filled yet.
    #[default]
    InProgress,
    /// Puzzle is 100% filled but has incorrect letters.
    IncorrectFill,
    /// Puzzle is 100% filled and all letters are correct.
    Correct,
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
    /// Whether to use "latest" puzzle instead of specific date.
    pub use_latest: bool,
    /// Currently selected provider index.
    pub provider_idx: usize,
    /// Which field is currently active.
    pub active_field: SelectionField,
    /// Error message to display, if any.
    pub error: Option<String>,
}

impl Default for SelectionState {
    fn default() -> Self {
        // Default to "latest" mode
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        Self {
            date: today,
            use_latest: true,
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

    /// Current completion state of the puzzle.
    pub completion_state: CompletionState,

    /// Final completion time (set when puzzle is completed correctly).
    pub completion_time: Option<Duration>,

    /// Selected option in the completion popup (0 = Continue, 1 = Menu).
    pub completed_popup_selection: usize,

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
            completion_state: CompletionState::default(),
            completion_time: None,
            completed_popup_selection: 0,
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
        self.completion_state = CompletionState::default();
        self.completion_time = None;
        self.completed_popup_selection = 0;
        self.scroll_cur = (0, 0);
        self.scroll_max = (0, 0);
        self.scroll_bar = (ScrollbarState::default(), ScrollbarState::default());
    }
}

impl App {
    pub fn draw_game(&mut self, view: GameView, frame: &mut ratatui::Frame) {
        match view {
            GameView::Playing => self.draw_game_playing(frame, false),
            GameView::CompletedPlaying => self.draw_game_playing(frame, true),
            GameView::Selecting => self.draw_game_selecting(frame),
            GameView::Loading => self.draw_game_loading(frame),
            GameView::Completed => self.draw_game_completed(frame),
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
        let date_title = if selection.use_latest {
            " Date "
        } else {
            " Date (YYYY-MM-DD) "
        };
        let date_block = Block::default()
            .title(date_title)
            .borders(Borders::ALL)
            .border_style(date_style);
        let date_inner = date_block.inner(rows[0]);
        frame.render_widget(date_block, rows[0]);

        let date_text = if selection.use_latest {
            "< Latest >".to_string()
        } else {
            let cursor_char = if selection.active_field == SelectionField::Date {
                "_"
            } else {
                ""
            };
            format!("< {} >", format!("{}{}", selection.date, cursor_char))
        };
        frame.render_widget(
            Paragraph::new(date_text).style(date_style).centered(),
            date_inner,
        );

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

        let provider_name = PuzzleProvider::ALL
            .get(self.state.game.selection.provider_idx)
            .map(|p| p.name())
            .unwrap_or("Unknown");

        let loading_text = if self.state.game.selection.use_latest {
            format!("Loading latest {} puzzle...", provider_name)
        } else {
            format!(
                "Loading {} puzzle for {}...",
                provider_name, self.state.game.selection.date
            )
        };
        frame.render_widget(
            Paragraph::new(loading_text)
                .style(Style::default().fg(Color::Cyan))
                .centered(),
            content_area,
        );
    }

    fn draw_game_playing(&mut self, frame: &mut ratatui::Frame, is_completed: bool) {
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

        // Total content: header (3) + padding (1) + grid + padding (1) + footer (3)
        let total_height = 3 + 1 + grid_content_height + 1 + 3;
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

        // Split into 5 areas: top bar, padding, grid, padding, bottom bar
        let layout = Layout::vertical([
            Constraint::Length(3), // top bar with borders
            Constraint::Length(1), // padding
            Constraint::Min(1),    // grid area
            Constraint::Length(1), // padding
            Constraint::Length(3), // bottom bar with borders
        ])
        .split(centered_area);

        let top_area = layout[0];
        let grid_area = layout[2];
        let bottom_area = layout[4];

        // === TOP BAR ===
        self.draw_top_bar(frame, top_area, is_completed);

        // === GRID ===
        self.state.game.visible_area = (grid_area.width, grid_area.height);

        let mut par = grid.to_par();
        let (width, height) = (grid_area.width, grid_area.height);

        // calculate vertical scroll bounds
        let content_height = (grid.height() as u16 * 4) + 1;
        let max_scroll_v = content_height.saturating_sub(height);

        // calculate horizontal scroll bounds
        let content_width = (grid.width() as u16 * 4) + 1;
        let max_scroll_h = content_width.saturating_sub(width);

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

    /// Draw the top bar: date (left), title (center), completion% + timer (right).
    fn draw_top_bar(&self, frame: &mut ratatui::Frame, area: Rect, is_completed: bool) {
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

        // Timer: use frozen completion_time if completed, otherwise live elapsed time
        let timer_duration = if is_completed {
            self.state.game.completion_time
        } else {
            self.state.game.start_time.map(|start| start.elapsed())
        };
        let timer_str = format_duration(timer_duration);

        // Completion percentage
        let (completion_str, completion_style) = if let Some(grid) = self.state.game.grid.as_ref() {
            let pct = grid.completion_percentage();
            let style = match self.state.game.completion_state {
                CompletionState::IncorrectFill => Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
                CompletionState::Correct => Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                CompletionState::InProgress => Style::default().fg(Color::White),
            };
            (format!("{}%", pct), style)
        } else {
            ("0%".to_string(), Style::default().fg(Color::White))
        };

        // Right side: "XX% MM:SS"
        let right_str = format!("{} {}", completion_str, timer_str);
        let right_len = right_str.len();

        // Calculate spacing for centering the title
        let total_width = inner.width as usize;
        let date_len = date_str.len();
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

        // Padding between title and right side (completion + timer)
        let pad_right = total_width
            .saturating_sub(right_start)
            .saturating_sub(right_len);
        if pad_right > 0 {
            spans.push(Span::raw(" ".repeat(pad_right)));
        }

        spans.push(Span::styled(completion_str, completion_style));
        spans.push(Span::raw(" "));
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

    /// Draw the completion popup overlay.
    fn draw_game_completed(&mut self, frame: &mut ratatui::Frame) {
        // First draw the game in the background (as completed)
        self.draw_game_playing(frame, true);

        let area = frame.area();

        // Popup dimensions
        let popup_width: u16 = 40;
        let popup_height: u16 = 9;

        // Center the popup
        let [centered_area] = Layout::horizontal([Constraint::Length(popup_width)])
            .flex(Flex::Center)
            .areas(area);

        let [centered_area] = Layout::vertical([Constraint::Length(popup_height)])
            .flex(Flex::Center)
            .areas(centered_area);

        // Clear the background area
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black)),
            centered_area,
        );

        // Draw popup border
        let block = Block::default()
            .title(" Congratulations! ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let inner_area = block.inner(centered_area);
        frame.render_widget(block, centered_area);

        // Build popup content
        let time_str = format_duration(self.state.game.completion_time);
        let selected = self.state.game.completed_popup_selection;

        let selected_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let normal_style = Style::default().fg(Color::White);

        let options = ["Continue Playing", "Back to Menu"];
        let option_lines = options.iter().enumerate().map(|(i, opt)| {
            let (prefix, style) = if i == selected {
                ("> ", selected_style)
            } else {
                ("  ", normal_style)
            };
            Line::from(Span::styled(format!("{}{}", prefix, opt), style))
        });

        let lines: Vec<Line> = [
            Line::from(""),
            Line::from(Span::styled(
                format!("Puzzle completed in {}!", time_str),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ]
        .into_iter()
        .chain(option_lines)
        .collect();

        frame.render_widget(Paragraph::new(lines).centered(), inner_area);
    }

    /// Get the clue text for the currently selected cell based on active direction.
    fn get_current_clue(&self) -> String {
        self.get_current_clue_inner().unwrap_or_default()
    }

    fn get_current_clue_inner(&self) -> Option<String> {
        let grid = self.state.game.grid.as_ref()?;
        let puzzle = self.state.game.puzzle.as_ref()?;
        let (row, col) = self.state.game.sel;
        let cell = grid.get(row, col)?;

        let direction = self.state.game.active_direction;

        // Try active direction first, fall back to the other direction
        let (clue_no, effective_dir) = cell
            .clue_no_for_direction(direction)
            .map(|n| (n, direction))
            .or_else(|| {
                let other = direction.toggle();
                cell.clue_no_for_direction(other).map(|n| (n, other))
            })?;

        let (dir_char, clue_text) = match effective_dir {
            Direction::Across => (
                'A',
                puzzle.clues.across.get(&(clue_no as u16)).map(String::as_str).unwrap_or("?"),
            ),
            Direction::Down => (
                'D',
                puzzle.clues.down.get(&(clue_no as u16)).map(String::as_str).unwrap_or("?"),
            ),
        };

        Some(format!("{}{}: {}", clue_no, dir_char, clue_text))
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
            GameView::Completed => self.handle_completed_input(key),
            GameView::CompletedPlaying => self.handle_completed_playing_input(key),
            GameView::Saving => {}
        }
    }

    fn handle_completed_input(&mut self, key: KeyEvent) {
        use crate::AppView;

        match key.code {
            KeyCode::Up => {
                if self.state.game.completed_popup_selection > 0 {
                    self.state.game.completed_popup_selection -= 1;
                }
            }
            KeyCode::Down => {
                if self.state.game.completed_popup_selection < 1 {
                    self.state.game.completed_popup_selection += 1;
                }
            }
            KeyCode::Enter => {
                match self.state.game.completed_popup_selection {
                    0 => {
                        // Continue Playing
                        self.view = AppView::Game(GameView::CompletedPlaying);
                    }
                    1 => {
                        // Back to Menu
                        self.view = AppView::Menu;
                    }
                    _ => {}
                }
            }
            KeyCode::Esc => {
                // ESC also goes back to menu
                self.view = AppView::Menu;
            }
            _ => {}
        }
    }

    fn handle_completed_playing_input(&mut self, key: KeyEvent) {
        use crate::AppView;

        match key.code {
            KeyCode::Esc => self.view = AppView::Menu,
            KeyCode::Char(' ') => self.toggle_direction(),
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                self.handle_arrow_navigation(key);
            }
            _ => {}
        }
    }

    /// Handle arrow key navigation with optional SHIFT modifier for word jumping.
    fn handle_arrow_navigation(&mut self, key: KeyEvent) {
        let (row_delta, col_delta) = match key.code {
            KeyCode::Up => (-1, 0),
            KeyCode::Down => (1, 0),
            KeyCode::Left => (0, -1),
            KeyCode::Right => (0, 1),
            _ => return,
        };

        if key.modifiers.contains(KeyModifiers::SHIFT) {
            self.jump_to_next_word(row_delta, col_delta);
        } else {
            self.move_selection(row_delta, col_delta);
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
                match self.state.game.selection.active_field {
                    SelectionField::Date => {
                        // Toggle between "Latest" and date input
                        self.state.game.selection.use_latest =
                            !self.state.game.selection.use_latest;
                    }
                    SelectionField::Provider => {
                        let len = PuzzleProvider::ALL.len();
                        if self.state.game.selection.provider_idx == 0 {
                            self.state.game.selection.provider_idx = len - 1;
                        } else {
                            self.state.game.selection.provider_idx -= 1;
                        }
                    }
                    _ => {}
                }
            }

            KeyCode::Right => {
                match self.state.game.selection.active_field {
                    SelectionField::Date => {
                        // Toggle between "Latest" and date input
                        self.state.game.selection.use_latest =
                            !self.state.game.selection.use_latest;
                    }
                    SelectionField::Provider => {
                        let len = PuzzleProvider::ALL.len();
                        self.state.game.selection.provider_idx =
                            (self.state.game.selection.provider_idx + 1) % len;
                    }
                    _ => {}
                }
            }

            KeyCode::Enter => {
                if self.state.game.selection.active_field == SelectionField::Start {
                    // Validate date format before starting (skip if using "Latest")
                    if self.state.game.selection.use_latest || self.validate_date() {
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
                let is_date_field = self.state.game.selection.active_field == SelectionField::Date;
                let is_not_latest = !self.state.game.selection.use_latest;
                let is_valid_char = c.is_ascii_digit() || c == '-';
                let has_room = self.state.game.selection.date.len() < 10;

                if is_date_field && is_not_latest && is_valid_char && has_room {
                    self.state.game.selection.date.push(c);
                    self.state.game.selection.error = None;
                }
            }

            KeyCode::Backspace => {
                let is_date_field = self.state.game.selection.active_field == SelectionField::Date;
                let is_not_latest = !self.state.game.selection.use_latest;

                if is_date_field && is_not_latest {
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
                self.check_completion();
                return;
            }
            // ALT+CTRL+R: reveal entire puzzle
            else if key.modifiers.contains(KeyModifiers::ALT) {
                if let Some(grid) = self.state.game.grid.as_mut() {
                    grid.reveal_all();
                }
                self.check_completion();
                return;
            }
            // CTRL+R: reveal current letter
            else {
                self.reveal_current_letter();
                self.advance_to_next_cell();
                self.check_completion();
                return;
            }
        } else if is_ctrl_r_char {
            // CTRL+R as control character: reveal current letter
            self.reveal_current_letter();
            self.advance_to_next_cell();
            self.check_completion();
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

            // Navigation: arrow keys move selection, SHIFT+arrow jumps to next word
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                self.handle_arrow_navigation(key);
            }

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
                // check completion after entering a letter
                self.check_completion();
            }

            // backspace/delete: clear the current cell and retreat
            KeyCode::Backspace | KeyCode::Delete => {
                let (row, col) = self.state.game.sel;
                if let Some(grid) = self.state.game.grid.as_mut() {
                    if let Some(cell) = grid.get_mut(row, col) {
                        cell.set_user_letter(None);
                    }
                }
                // move to previous cell in active direction
                self.retreat_to_prev_cell();
                // update completion state after clearing
                self.update_completion_state();
            }

            _ => {}
        }
    }

    /// Check completion state and transition to Completed view if puzzle is solved.
    fn check_completion(&mut self) {
        self.update_completion_state();

        // If puzzle is fully correct, transition to Completed view
        if self.state.game.completion_state == CompletionState::Correct {
            use crate::AppView;

            // Store the completion time
            if let Some(start) = self.state.game.start_time {
                self.state.game.completion_time = Some(start.elapsed());
            }

            // Reset popup selection
            self.state.game.completed_popup_selection = 0;

            // Transition to Completed view
            self.view = AppView::Game(GameView::Completed);
        }
    }

    /// Update completion state based on current grid fill.
    fn update_completion_state(&mut self) {
        let Some(grid) = self.state.game.grid.as_ref() else {
            return;
        };

        let percentage = grid.completion_percentage();

        if percentage < 100 {
            self.state.game.completion_state = CompletionState::InProgress;
        } else if grid.is_fully_correct() {
            self.state.game.completion_state = CompletionState::Correct;
        } else {
            self.state.game.completion_state = CompletionState::IncorrectFill;
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

        let clue_no = self
            .state
            .game
            .grid
            .as_ref()
            .and_then(|grid| grid.get(row, col))
            .and_then(|cell| cell.clue_no_for_direction(direction));

        if let (Some(clue_no), Some(grid)) = (clue_no, self.state.game.grid.as_mut()) {
            grid.reveal_word(clue_no, direction);
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

    /// Jump to the first empty cell in a different clue word in the given direction.
    ///
    /// Scans in the given direction until finding an empty cell that belongs to a different
    /// clue word. Wraps rows/columns as needed. If nothing is found, selects the last
    /// non-filled cell encountered.
    fn jump_to_next_word(&mut self, row_delta: i32, col_delta: i32) {
        let Some(grid) = self.state.game.grid.as_ref() else {
            return;
        };

        let direction = self.state.game.active_direction;
        let (cur_row, cur_col) = self.state.game.sel;
        let height = grid.height() as i32;
        let width = grid.width() as i32;

        // Get the current cell's clue number for the movement direction
        // For horizontal movement (left/right), use Across clue
        // For vertical movement (up/down), use Down clue
        let movement_direction = if col_delta != 0 {
            Direction::Across
        } else {
            Direction::Down
        };

        let current_clue = grid
            .get(cur_row, cur_col)
            .and_then(|c| c.clue_no_for_direction(movement_direction));

        let mut new_row = cur_row as i32 + row_delta;
        let mut new_col = cur_col as i32 + col_delta;

        // Track the target cell (empty cell in different word) and fallback (last non-filled)
        let mut target: Option<(usize, usize)> = None;
        let mut last_valid: Option<(usize, usize)> = None;
        let start = (cur_row, cur_col);
        let mut wrapped = false;

        loop {
            // Handle wrapping at grid boundaries
            if new_row < 0 || new_row >= height || new_col < 0 || new_col >= width {
                // Wrap based on movement direction
                if col_delta != 0 {
                    // Horizontal movement: wrap to next/prev row
                    new_row += col_delta.signum();
                    new_col = if col_delta > 0 { 0 } else { width - 1 };
                    if new_row < 0 {
                        new_row = height - 1;
                    } else if new_row >= height {
                        new_row = 0;
                    }
                } else {
                    // Vertical movement: wrap to next/prev column
                    new_col += row_delta.signum();
                    new_row = if row_delta > 0 { 0 } else { height - 1 };
                    if new_col < 0 {
                        new_col = width - 1;
                    } else if new_col >= width {
                        new_col = 0;
                    }
                }
                wrapped = true;
            }

            // Check if we've come back to start
            if wrapped && (new_row as usize, new_col as usize) == start {
                break;
            }

            if let Some(cell) = grid.get(new_row as usize, new_col as usize) {
                if !cell.is_filled() {
                    // Track as potential fallback
                    last_valid = Some((new_row as usize, new_col as usize));

                    // Check if this cell belongs to a different clue word
                    let cell_clue = cell.clue_no_for_direction(movement_direction);
                    let is_different_word = match (current_clue, cell_clue) {
                        (Some(cur), Some(c)) => cur != c,
                        (None, Some(_)) => true,
                        _ => false,
                    };

                    // Found an empty cell in a different word - this is our target
                    if is_different_word && cell.is_empty() {
                        target = Some((new_row as usize, new_col as usize));
                        break;
                    }
                }
            }

            // Continue in the same direction
            new_row += row_delta;
            new_col += col_delta;
        }

        // Now apply the selection (after releasing the immutable borrow)
        let final_target = target.or(last_valid);
        if let Some((row, col)) = final_target {
            if let Some(grid) = self.state.game.grid.as_mut() {
                if grid.set_selection(row, col, direction) {
                    self.state.game.sel = (row, col);
                }
            }
            self.ensure_selection_visible();
        }
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

    /// Retreat to the previous cell in the active direction (opposite of advance).
    ///
    /// - If direction is `Across`: move left, wrap to previous row if at start
    /// - If direction is `Down`: move up, wrap to previous column if at start
    fn retreat_to_prev_cell(&mut self) {
        let Some(grid) = self.state.game.grid.as_mut() else {
            return;
        };

        let direction = self.state.game.active_direction;
        let (cur_row, cur_col) = self.state.game.sel;
        let height = grid.height() as i32;
        let width = grid.width() as i32;

        // determine the delta based on direction (opposite of advance)
        let (row_delta, col_delta): (i32, i32) = match direction {
            Direction::Across => (0, -1),
            Direction::Down => (-1, 0),
        };

        let mut new_row = cur_row as i32 + row_delta;
        let mut new_col = cur_col as i32 + col_delta;

        // try to find the previous non-filled cell
        loop {
            // check if we've gone past the grid boundary
            if new_row < 0 || new_row >= height || new_col < 0 || new_col >= width {
                // wrap to previous row/column
                match direction {
                    Direction::Across => {
                        new_row -= 1;
                        new_col = width - 1;
                    }
                    Direction::Down => {
                        new_row = height - 1;
                        new_col -= 1;
                    }
                }

                // if we've wrapped past the entire grid, stop
                if new_row < 0 || new_col < 0 {
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

            // move to previous cell
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
