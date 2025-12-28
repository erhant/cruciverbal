use crate::{App, AppView};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout};
mod grid;
use grid::*;

#[derive(Default, Debug)]
pub struct GameState {
    pub puzzle: Option<puz_parse::Puzzle>,
}

impl App {
    pub fn draw_game(&mut self, frame: &mut ratatui::Frame) {
        let Some(puzzle) = self.state.game.puzzle.as_ref() else {
            return; // nothing to draw
        };

        let area = frame.area();
        let col_constraints = (0..puzzle.info.width).map(|_| Constraint::Length(5));
        let row_constraints = (0..puzzle.info.height).map(|_| Constraint::Length(5));

        // spacing with overlap so that borders are joined
        let horizontal = Layout::horizontal(col_constraints);
        let vertical = Layout::vertical(row_constraints);

        let rows = vertical.split(area);
        let grid_areas = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());
        let puzzle_cells = puzzle
            .grid
            .solution
            .join("")
            .chars()
            .map(|c| {
                if c == '.' {
                    PuzzleCell::new_filled()
                } else {
                    PuzzleCell::new_valued(c)
                }
            })
            .collect::<Vec<PuzzleCell>>();
        assert!(puzzle_cells.len() == puzzle.info.width as usize * puzzle.info.height as usize);

        // for (cell, area) in puzzle_cells.iter().zip(grid_areas) {
        //     frame.render_widget(cell.to_par(), area);
        // }
    }

    pub fn handle_game_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.is_running = false;
            }
            _ => {}
        }
    }
}
