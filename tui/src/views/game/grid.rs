use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::constants::*;
use super::{ClueNoDirection, Direction, PuzzleCell, WordIdxDirection};

/// A grid of cells.
#[derive(Debug)]
pub struct PuzzleGrid {
    cells: Vec<Vec<PuzzleCell>>,
}

impl PuzzleGrid {
    pub fn cells(&self) -> &[Vec<PuzzleCell>] {
        &self.cells
    }

    pub fn cells_mut(&mut self) -> &mut [Vec<PuzzleCell>] {
        &mut self.cells
    }

    /// Get a reference to a cell at the given position.
    pub fn get(&self, row: usize, col: usize) -> Option<&PuzzleCell> {
        self.cells.get(row).and_then(|r| r.get(col))
    }

    /// Get a mutable reference to a cell at the given position.
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut PuzzleCell> {
        self.cells.get_mut(row).and_then(|r| r.get_mut(col))
    }

    /// Update the selection to the given cell, clearing all previous selections.
    ///
    /// This sets `is_selected_cell` on the new cell and `is_selected_word` on all
    /// other cells in the same word (based on the active direction).
    ///
    /// Returns `true` if the selection was updated (cell exists and is not filled).
    pub fn set_selection(&mut self, new_row: usize, new_col: usize, direction: Direction) -> bool {
        // Check if the new cell is valid and not filled
        let clue_no = if let Some(new_cell) = self.get(new_row, new_col) {
            if new_cell.is_filled() {
                return false;
            }
            // Get the clue number for the active direction
            new_cell.clue_no_for_direction(direction)
        } else {
            return false;
        };

        // Clear all selections first
        for row in self.cells.iter_mut() {
            for cell in row.iter_mut() {
                cell.clear_selection();
            }
        }

        // Set the cell selection
        if let Some(new_cell) = self.get_mut(new_row, new_col) {
            new_cell.is_selected_cell = true;
        }

        // Highlight the word if we have a clue number for this direction
        if let Some(target_clue) = clue_no {
            for row in self.cells.iter_mut() {
                for cell in row.iter_mut() {
                    if cell.is_selected_cell {
                        continue; // Don't mark the cursor cell as word-selected
                    }
                    if let Some(cell_clue) = cell.clue_no_for_direction(direction) {
                        if cell_clue == target_clue {
                            cell.is_selected_word = true;
                        }
                    }
                }
            }
        }

        true
    }

    /// Clear all cell and word selections in the grid.
    pub fn clear_all_selections(&mut self) {
        self.cells
            .iter_mut()
            .flat_map(|row| row.iter_mut())
            .for_each(|cell| cell.clear_selection());
    }

    /// Reveal all cells in a word with the given clue number and direction.
    ///
    /// This sets each cell's user_letter to its clue_letter.
    pub fn reveal_word(&mut self, clue_no: usize, direction: Direction) {
        self.cells
            .iter_mut()
            .flat_map(|row| row.iter_mut())
            .filter(|cell| cell.clue_no_for_direction(direction) == Some(clue_no))
            .for_each(|cell| cell.reveal());
    }

    /// Reveal all cells in the grid.
    pub fn reveal_all(&mut self) {
        self.cells
            .iter_mut()
            .flat_map(|row| row.iter_mut())
            .for_each(|cell| cell.reveal());
    }

    /// Find the first non-filled cell in the grid (for initial selection).
    pub fn find_first_letter_cell(&self) -> Option<(usize, usize)> {
        for (row_idx, row) in self.cells.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                if !cell.is_filled() {
                    return Some((row_idx, col_idx));
                }
            }
        }
        None
    }

    /// Count the total number of letter cells (excluding filled/black cells).
    pub fn count_total_letters(&self) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| !cell.is_filled())
            .count()
    }

    /// Count the number of letter cells that have a user-entered letter.
    pub fn count_filled_letters(&self) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.get_user_letter().is_some())
            .count()
    }

    /// Check if all letter cells have been filled correctly.
    ///
    /// Returns `true` if every letter cell has a user_letter that matches clue_letter.
    pub fn is_fully_correct(&self) -> bool {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| !cell.is_filled())
            .all(|cell| cell.is_correct() == Some(true))
    }

    /// Get completion percentage (0-100).
    pub fn completion_percentage(&self) -> u8 {
        let total = self.count_total_letters();
        if total == 0 {
            return 100;
        }
        let filled = self.count_filled_letters();
        ((filled * 100) / total) as u8
    }

    /// Create a new [`PuzzleGrid`] from a 2D vector of [`PuzzleCell`]s.
    pub fn new(cells: Vec<Vec<PuzzleCell>>) -> Self {
        assert!(!cells.is_empty());

        // sanity check: all rows must have the same length
        let row_len = cells[0].len();
        for row in &cells {
            assert!(row.len() == row_len, "All rows must have the same length");
        }

        Self { cells }
    }

    /// Width of the grid in number of cells, as `u8` for `.puz` compatibility.
    pub fn width(&self) -> u8 {
        self.cells[0].len() as u8
    }

    /// Height of the grid in number of cells, as `u8` for `.puz` compatibility.
    pub fn height(&self) -> u8 {
        self.cells.len() as u8
    }

    /// Build a [`PuzzleGrid`] from a [`puz_parse`] solution grid.
    ///
    /// Uses standard crossword numbering: scan left-to-right, top-to-bottom.
    ///
    /// - A cell gets a number if it starts an across word OR a down word.
    /// - A word "starts" if the cell to the left/above is filled (or at edge)
    ///   AND there's at least one more letter to the right/below.
    pub fn from_solution(solution: &[String]) -> Self {
        let height = solution.len();
        if height == 0 {
            panic!("Solution grid cannot be empty");
        }
        let width = solution[0].chars().count();

        // convert solution to char grid for easier indexing
        let chars: Vec<Vec<char>> = solution.iter().map(|row| row.chars().collect()).collect();

        // helper to check if a cell is filled (black square)
        let is_filled = |r: usize, c: usize| chars[r][c] == '.';

        // helper to check if cell is in bounds and is a letter
        let is_letter = |r: i32, c: i32| -> bool {
            r >= 0
                && c >= 0
                && (r as usize) < height
                && (c as usize) < width
                && !is_filled(r as usize, c as usize)
        };

        // Pass 1: Number cells, track which cells start across/down words
        let mut across_clue_at: Vec<Vec<Option<usize>>> = vec![vec![None; width]; height];
        let mut down_clue_at: Vec<Vec<Option<usize>>> = vec![vec![None; width]; height];
        let mut current_number = 1usize;

        for row in 0..height {
            for col in 0..width {
                if is_filled(row, col) {
                    continue;
                }

                // Starts across: no letter to left AND letter to right
                let starts_across =
                    !is_letter(row as i32, col as i32 - 1) && is_letter(row as i32, col as i32 + 1);
                // Starts down: no letter above AND letter below
                let starts_down =
                    !is_letter(row as i32 - 1, col as i32) && is_letter(row as i32 + 1, col as i32);

                if starts_across || starts_down {
                    if starts_across {
                        across_clue_at[row][col] = Some(current_number);
                    }
                    if starts_down {
                        down_clue_at[row][col] = Some(current_number);
                    }
                    current_number += 1;
                }
            }
        }

        // Pass 2: Build cells with word_idx and clue_no
        let mut cells: Vec<Vec<PuzzleCell>> = Vec::with_capacity(height);

        for row in 0..height {
            let mut row_cells = Vec::with_capacity(width);
            for col in 0..width {
                if is_filled(row, col) {
                    row_cells.push(PuzzleCell::filled());
                    continue;
                }

                let clue_letter = chars[row][col];

                // find across word: scan left to start, get clue# and compute offset
                let across_info = if is_letter(row as i32, col as i32 - 1)
                    || is_letter(row as i32, col as i32 + 1)
                {
                    let mut start = col;
                    while start > 0 && !is_filled(row, start - 1) {
                        start -= 1;
                    }
                    across_clue_at[row][start].map(|n| (n, col - start))
                } else {
                    None
                };

                // find down word: scan up to start, get clue# and compute offset
                let down_info = if is_letter(row as i32 - 1, col as i32)
                    || is_letter(row as i32 + 1, col as i32)
                {
                    let mut start = row;
                    while start > 0 && !is_filled(start - 1, col) {
                        start -= 1;
                    }
                    down_clue_at[start][col].map(|n| (n, row - start))
                } else {
                    None
                };

                #[rustfmt::skip]
                let (word_idx, clue_no) = match (across_info, down_info) {
                    (Some((a, ai)), Some((d, di))) => (
                        WordIdxDirection::Cross(ai, di),
                        ClueNoDirection::Cross(a, d),
                    ),
                    (Some((a, ai)), None) => (
                        WordIdxDirection::Across(ai),
                        ClueNoDirection::Across(a)
                    ),
                    (None, Some((d, di))) => (
                        WordIdxDirection::Down(di),
                        ClueNoDirection::Down(d)
                    ),
                    (None, None) => panic!("Isolated cell at ({}, {})", row, col),
                };

                row_cells.push(PuzzleCell::valued(clue_letter, word_idx, clue_no));
            }
            cells.push(row_cells);
        }

        Self::new(cells)
    }

    /// Convert a [`PuzzleCell`] grid to a [`Paragraph`] for rendering.
    ///
    /// Each cell is 4 characters wide by 4 lines tall. Adjacent cells share borders,
    /// so we only draw the left and top borders for each cell, plus the right and
    /// bottom borders for the last column/row.
    pub fn to_par(&self) -> Paragraph {
        let num_rows = self.cells.len();
        let num_cols = self.cells[0].len();
        let border_style = Style::default().fg(Color::White);

        // Helper closures for common spans
        let h_span = || Span::styled(BOX_H.to_string(), border_style);
        let v_span = || Span::styled(BOX_V.to_string(), border_style);
        let empty = || Span::raw(" ");
        let corner = |c: char| Span::styled(c.to_string(), border_style);

        let mut all_lines: Vec<Line> = Vec::new();

        for (row_idx, cell_row) in self.cells.iter().enumerate() {
            let is_first_row = row_idx == 0;
            let is_last_row = row_idx == num_rows - 1;

            // Each cell row produces 4 lines (5 for last row to include bottom border)
            let num_lines = if is_last_row { 5 } else { 4 };
            let mut span_groups: Vec<Vec<Span>> = vec![Vec::new(); num_lines];

            for (col_idx, cell) in cell_row.iter().enumerate() {
                let is_first_col = col_idx == 0;
                let is_last_col = col_idx == num_cols - 1;

                let val_span = cell.to_val_span();
                let selection_span = cell.to_selection_span();
                let (no_span_1, no_span_2) = cell.to_no_spans(border_style);

                // Top-left corner: depends on position in grid
                let tl_corner = match (is_first_row, is_first_col) {
                    (true, true) => BOX_TL,
                    (true, false) => BOX_T,
                    (false, true) => BOX_L,
                    (false, false) => BOX_X,
                };
                span_groups[0].extend([corner(tl_corner), no_span_1, no_span_2, h_span()]);

                // Top-right corner for last column
                if is_last_col {
                    let tr_corner = if is_first_row { BOX_TR } else { BOX_R };
                    span_groups[0].push(corner(tr_corner));
                }

                // Content lines (3 lines with left border)
                if cell.is_filled() {
                    // Filled cells: solid block across all 3 interior positions
                    let filled = || Span::styled(BOX_FILLED.to_string(), Style::default().fg(Color::White));
                    span_groups[1].extend([v_span(), filled(), filled(), filled()]);
                    span_groups[2].extend([v_span(), filled(), filled(), filled()]);
                    span_groups[3].extend([v_span(), filled(), filled(), filled()]);
                } else {
                    // Letter cells: normal layout with value in center
                    span_groups[1].extend([v_span(), empty(), empty(), empty()]);
                    span_groups[2].extend([v_span(), empty(), val_span, empty()]);
                    span_groups[3].extend([v_span(), empty(), selection_span, empty()]);
                }

                // Right border for last column
                if is_last_col {
                    span_groups[1].push(v_span());
                    span_groups[2].push(v_span());
                    span_groups[3].push(v_span());
                }

                // Bottom border for last row
                if is_last_row {
                    let bl_corner = if is_first_col { BOX_BL } else { BOX_B };
                    span_groups[4].extend([corner(bl_corner), h_span(), h_span(), h_span()]);

                    if is_last_col {
                        span_groups[4].push(corner(BOX_BR));
                    }
                }
            }

            for spans in span_groups {
                all_lines.push(Line::from_iter(spans));
            }
        }

        Paragraph::new(all_lines)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::widgets::Widget;

    use crate::game::cell::{ClueNoDirection, PuzzleCellValue, WordIdxDirection};

    use super::*;

    impl PuzzleCell {
        pub fn cheated(
            clue_letter: char,
            word_idx: WordIdxDirection,
            clue_no: ClueNoDirection,
        ) -> Self {
            Self {
                val: PuzzleCellValue::Letter {
                    clue_letter,
                    user_letter: Some(clue_letter),
                    word_idx,
                    clue_no,
                },
                is_selected_cell: false,
                is_selected_word: false,
            }
        }
    }

    #[test]
    fn test_from_solution() {
        // . . B     <- B starts down word (clue 1)
        // A C E     <- A starts across word (clue 2), E is cross (across 2, down 1)
        // . . E
        #[rustfmt::skip]
        let solution = vec![
          "..B".to_string(),
          "ACE".to_string(),
          "..E".to_string()
        ];

        let grid = PuzzleGrid::from_solution(&solution);
        println!("{:#?}", grid);
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 3);

        let cells = grid.cells();

        // (0,0) and (0,1) are filled
        assert!(matches!(cells[0][0].val, PuzzleCellValue::Filled));
        assert!(matches!(cells[0][1].val, PuzzleCellValue::Filled));

        // (0,2) is 'B', starts down clue 1, position 0 in down word
        if let PuzzleCellValue::Letter {
            clue_letter,
            word_idx,
            clue_no,
            ..
        } = &cells[0][2].val
        {
            assert_eq!(*clue_letter, 'B');
            assert!(matches!(word_idx, WordIdxDirection::Down(0)));
            assert!(matches!(clue_no, ClueNoDirection::Down(1)));
        } else {
            panic!("Expected letter cell at (0,2)");
        }

        // (1,0) is 'A', starts across clue 2, position 0 in across word
        if let PuzzleCellValue::Letter {
            clue_letter,
            word_idx,
            clue_no,
            ..
        } = &cells[1][0].val
        {
            assert_eq!(*clue_letter, 'A');
            assert!(matches!(word_idx, WordIdxDirection::Across(0)));
            assert!(matches!(clue_no, ClueNoDirection::Across(2)));
        } else {
            panic!("Expected letter cell at (1,0)");
        }

        // (1,1) is 'C', position 1 in across word (clue 2), only across
        if let PuzzleCellValue::Letter {
            clue_letter,
            word_idx,
            clue_no,
            ..
        } = &cells[1][1].val
        {
            assert_eq!(*clue_letter, 'C');
            assert!(matches!(word_idx, WordIdxDirection::Across(1)));
            assert!(matches!(clue_no, ClueNoDirection::Across(2)));
        } else {
            panic!("Expected letter cell at (1,1)");
        }

        // (1,2) is 'E', crossing: across pos 2 (clue 2), down pos 1 (clue 1)
        if let PuzzleCellValue::Letter {
            clue_letter,
            word_idx,
            clue_no,
            ..
        } = &cells[1][2].val
        {
            assert_eq!(*clue_letter, 'E');
            assert!(matches!(word_idx, WordIdxDirection::Cross(2, 1)));
            assert!(matches!(clue_no, ClueNoDirection::Cross(2, 1)));
        } else {
            panic!("Expected letter cell at (1,2)");
        }

        // (2,2) is 'E', position 2 in down word (clue 1)
        if let PuzzleCellValue::Letter {
            clue_letter,
            word_idx,
            clue_no,
            ..
        } = &cells[2][2].val
        {
            assert_eq!(*clue_letter, 'E');
            assert!(matches!(word_idx, WordIdxDirection::Down(2)));
            assert!(matches!(clue_no, ClueNoDirection::Down(1)));
        } else {
            panic!("Expected letter cell at (2,2)");
        }
    }

    #[test]
    fn test_puzzle_cell_to_par() {
        type PC = PuzzleCell;
        type WI = WordIdxDirection;
        type CN = ClueNoDirection;

        #[rustfmt::skip]
        let cells = vec![
            vec![PC::filled(),        PC::filled(),    PC::cheated('B', WI::Down(0), CN::Down(1))],
            vec![PC::cheated('A', WI::Across(0), CN::Across(1) ), PC::valued('C', WI::Across(1), CN::Across(1)), PC::cheated('E', WI::Cross(2, 1), CN::Cross(1, 1)).as_selected()],
            vec![PC::filled(),       PC::filled(),    PC::cheated('E', WI::Down(2), CN::Down( 1))],
        ];

        let grid = PuzzleGrid::new(cells);
        let par = grid.to_par();

        // create a dummy area for rendering
        let (width, height) = (35, 15);
        let area = ratatui::layout::Rect {
            x: 0,
            y: 0,
            width,
            height,
        };
        let mut buffer = ratatui::buffer::Buffer::empty(area);
        par.render(area, &mut buffer);

        // print the buffer for visual inspection
        let mut out = String::new();
        for y in 0..height {
            let mut line = String::new();
            for x in 0..width {
                if let Some(cell) = buffer.cell((x, y)) {
                    line.push_str(cell.symbol());
                }
            }
            out.push_str(line.trim_end());
            out.push('\n');
        }

        println!("{out}");
    }
}
