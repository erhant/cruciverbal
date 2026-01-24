use crate::App;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::time::Instant;

mod boxchars;

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
            scroll_cur: (0, 0),
            scroll_max: (0, 0),
            scroll_bar: (ScrollbarState::default(), ScrollbarState::default()),
        }
    }
}

impl App {
    pub fn draw_game(&mut self, view: GameView, frame: &mut ratatui::Frame) {
        match view {
            GameView::Playing => self.draw_game_playing(frame),
            _ => todo!(),
        }
    }

    fn draw_game_playing(&mut self, frame: &mut ratatui::Frame) {
        // Initialize grid from puzzle if not already done
        if self.state.game.grid.is_none() {
            if let Some(puzzle) = self.state.game.puzzle.as_ref() {
                let mut grid = PuzzleGrid::from_solution(&puzzle.grid.solution);

                // Find and select the first letter cell
                if let Some((row, col)) = grid.find_first_letter_cell() {
                    grid.set_selection(row, col, self.state.game.active_direction);
                    self.state.game.sel = (row, col);
                }

                self.state.game.grid = Some(grid);

                // Start timer when grid is initialized
                if self.state.game.start_time.is_none() {
                    self.state.game.start_time = Some(Instant::now());
                }
            }
        }

        let Some(grid) = self.state.game.grid.as_ref() else {
            return; // nothing to draw
        };

        // Split into 3 areas: top bar, grid, bottom bar
        let full_area = frame.area();
        let layout = Layout::vertical([
            Constraint::Length(1), // top bar
            Constraint::Min(1),    // grid area
            Constraint::Length(1), // bottom bar (clue)
        ])
        .split(full_area);

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
        let total_width = area.width as usize;
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
        frame.render_widget(Paragraph::new(line), area);
    }

    /// Draw the clue bar based on currently selected cell.
    fn draw_clue_bar(&self, frame: &mut ratatui::Frame, area: Rect) {
        let clue_text = self.get_current_clue();
        let clue_style = Style::default().fg(Color::White);
        let par = Paragraph::new(Span::styled(clue_text, clue_style));
        frame.render_widget(par, area);
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

    pub fn handle_game_input(&mut self, _view: GameView, key: KeyEvent) {
        if self.state.game.grid.is_none() {
            return;
        }

        match key.code {
            // ESC: quit game
            KeyCode::Esc => {
                self.is_running = false;
            }

            // SPACEBAR: toggle direction (Across <-> Down)
            KeyCode::Char(' ') => {
                self.toggle_direction();
            }

            // Navigation: move selection with arrow keys
            KeyCode::Up => self.move_selection(-1, 0),
            KeyCode::Down => self.move_selection(1, 0),
            KeyCode::Left => self.move_selection(0, -1),
            KeyCode::Right => self.move_selection(0, 1),

            // Letter input: A-Z (and lowercase a-z)
            KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                let letter = c.to_ascii_uppercase();
                let (row, col) = self.state.game.sel;
                if let Some(grid) = self.state.game.grid.as_mut() {
                    if let Some(cell) = grid.get_mut(row, col) {
                        cell.set_user_letter(Some(letter));
                    }
                }
            }

            // Backspace/Delete: clear the current cell
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

        // Re-apply selection to update word highlighting
        let (row, col) = self.state.game.sel;
        let direction = self.state.game.active_direction;
        if let Some(grid) = self.state.game.grid.as_mut() {
            grid.set_selection(row, col, direction);
        }
    }

    /// Move selection by the given delta, skipping filled cells.
    fn move_selection(&mut self, row_delta: i32, col_delta: i32) {
        let direction = self.state.game.active_direction;

        let Some(grid) = self.state.game.grid.as_mut() else {
            return;
        };

        let (cur_row, cur_col) = self.state.game.sel;
        let height = grid.height() as i32;
        let width = grid.width() as i32;

        let mut new_row = cur_row as i32 + row_delta;
        let mut new_col = cur_col as i32 + col_delta;

        // Keep moving in the same direction until we find a non-filled cell or hit boundary
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
            // Skip filled cells, continue in same direction
            new_row += row_delta;
            new_col += col_delta;
        }

        // Now update scroll position (grid borrow is dropped)
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

        // Vertical scrolling
        let visible_top = scroll_v;
        let visible_bottom = scroll_v + visible_h.saturating_sub(2); // account for scrollbar

        let new_scroll_v = if cell_top < visible_top {
            // Cell is above visible area
            cell_top
        } else if cell_bottom > visible_bottom {
            // Cell is below visible area
            cell_bottom.saturating_sub(visible_h.saturating_sub(2))
        } else {
            scroll_v
        };

        // Horizontal scrolling
        let visible_left = scroll_h;
        let visible_right = scroll_h + visible_w.saturating_sub(2); // account for scrollbar

        let new_scroll_h = if cell_left < visible_left {
            // Cell is to the left of visible area
            cell_left
        } else if cell_right > visible_right {
            // Cell is to the right of visible area
            cell_right.saturating_sub(visible_w.saturating_sub(2))
        } else {
            scroll_h
        };

        self.state.game.scroll_cur = (
            new_scroll_v.min(self.state.game.scroll_max.0),
            new_scroll_h.min(self.state.game.scroll_max.1),
        );
    }
}
