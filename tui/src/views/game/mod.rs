use crate::App;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

mod boxchars;

mod grid;
use grid::*;

mod cell;
use cell::*;

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

#[derive(Default, Debug)]
pub struct GameState {
    /// Loaded puzzle, if any.
    pub puzzle: Option<puz_parse::Puzzle>,

    // TODO: add grid here & make it editable
    /// Selected cell.
    pub sel: (usize, usize),

    /* scrollbar stuff */
    /// Current scroll position (vertical, horizontal).
    pub scroll_cur: (u16, u16),
    /// Maximum scroll position (vertical, horizontal), be careful about this as it may crash the app
    /// if set incorrectly.
    pub scroll_max: (u16, u16),
    /// Scrollbar state for the [`Scrollbar`] widget (vertical, horizontal).
    pub scroll_bar: (ScrollbarState, ScrollbarState),
}

impl App {
    pub fn draw_game(&mut self, view: GameView, frame: &mut ratatui::Frame) {
        match view {
            GameView::Playing => self.draw_game_playing(frame),
            _ => todo!(),
        }
    }

    fn draw_game_playing(&mut self, frame: &mut ratatui::Frame) {
        let Some(puzzle) = self.state.game.puzzle.as_ref() else {
            return; // nothing to draw
        };

        let area = frame.area();
        let puzzle_grid = PuzzleGrid::from_solution(&puzzle.grid.solution);
        assert!(puzzle_grid.height() == puzzle.info.height);
        assert!(puzzle_grid.width() == puzzle.info.width);

        // // render the puzzle grid with scrollbars
        let mut par = puzzle_grid.to_par();
        let (width, height) = (area.width, area.height);

        // calculate vertical scroll bounds
        let num_lines = par.line_count(width.saturating_sub(2)); // account for vertical scrollbar
        let max_scroll_v = num_lines.saturating_sub(height.saturating_sub(1) as usize); // account for horizontal scrollbar

        // calculate horizontal scroll bounds
        // each cell is 5 chars wide with 1 overlap, so total width is `num_cols * 4 + 1` (for border)
        let content_width = (puzzle_grid.width() as usize * 4 + 1) as u16;
        let max_scroll_h = content_width.saturating_sub(width.saturating_sub(2)) as usize; // account for scrollbars

        self.state.game.scroll_max = (max_scroll_v as u16, max_scroll_h as u16);

        // sanity check
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
        frame.render_widget(par, area);

        // update and render vertical scrollbar
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
            area,
            &mut self.state.game.scroll_bar.0,
        );

        // update and render horizontal scrollbar
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
                .end_symbol(Some("->")),
            area,
            &mut self.state.game.scroll_bar.1,
        );
    }

    pub fn handle_game_input(&mut self, view: GameView, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.is_running = false;
            }
            // scroll up (vertical offset shrinks)
            KeyCode::Up => {
                if self.state.game.scroll_cur.0 > 0 {
                    self.state.game.scroll_cur.0 -= 1;
                }
            }
            // scroll down (vertical offset grows)
            KeyCode::Down => {
                if self.state.game.scroll_cur.0 < self.state.game.scroll_max.0 {
                    self.state.game.scroll_cur.0 += 1;
                }
            }
            // scroll left (horizontal offset shrinks)
            KeyCode::Left => {
                if self.state.game.scroll_cur.1 > 0 {
                    self.state.game.scroll_cur.1 -= 1;
                }
            }
            // scroll right (horizontal offset grows)
            KeyCode::Right => {
                if self.state.game.scroll_cur.1 < self.state.game.scroll_max.1 {
                    self.state.game.scroll_cur.1 += 1;
                }
            }
            _ => {}
        }
    }
}
