use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

// Box drawing characters
const BOX_H: char = '═';
const BOX_V: char = '║';
const BOX_TL: char = '╔';
const BOX_TR: char = '╗';
const BOX_BL: char = '╚';
const BOX_BR: char = '╝';
const BOX_T: char = '╦';
const BOX_B: char = '╩';
const BOX_L: char = '╠';
const BOX_R: char = '╣';
const BOX_X: char = '╬';

const BOX_EMPTY: char = ' ';
const BOX_FILLED: char = '█';

/// A cell in the puzzle grid.
///
/// It is to be rendered as follows:
///
/// ```txt
/// border   no'    no'' border border
/// border                      border
/// border          val         border
/// border           _*         border
/// border border border border border
/// ```
///
/// Legend:
/// - `*`: if selected only
/// - `'`: optional, if numbered
/// - `''`: optional, if numbered and two-digit
///
/// Note that borders are shared between adjacent cells, and so
/// the [grid](PuzzleGrid) rendering logic takes care to only draw them once.
pub struct PuzzleCell {
    /// Number of the clue starting at this cell, if any.
    pub no: Option<u16>,
    /// Value contained within this cell.
    pub val: PuzzleCellValue,
    /// Whether this cell is currently selected by the user.
    pub is_selected: bool,
}

impl From<char> for PuzzleCell {
    fn from(c: char) -> Self {
        Self {
            no: None,
            val: PuzzleCellValue::from(c),
            is_selected: false,
        }
    }
}

impl From<char> for PuzzleCellValue {
    fn from(c: char) -> Self {
        // `.` is empty cell in `.puz` format,
        // and we treat space as empty cell too
        if c == '.' || c == ' ' {
            PuzzleCellValue::Empty
        } else if c.is_ascii_alphabetic() {
            PuzzleCellValue::Letter(c)
        } else {
            // anything non-empty & non-letter is treated as filled (black) cell
            PuzzleCellValue::Filled
        }
    }
}

pub enum PuzzleCellValue {
    Empty,        // empty cell
    Filled,       // black cell
    Letter(char), // letter cell
}

impl PuzzleCell {
    /// Create a new empty cell.
    pub fn empty() -> Self {
        Self {
            no: None,
            val: PuzzleCellValue::Empty,
            is_selected: false,
        }
    }

    /// Check if the cell is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self.val, PuzzleCellValue::Empty)
    }

    /// Create a new filled (black) cell.
    pub fn filled() -> Self {
        Self {
            no: None,
            val: PuzzleCellValue::Filled,
            is_selected: false,
        }
    }

    /// Check if the cell is filled (black).
    pub fn is_filled(&self) -> bool {
        matches!(self.val, PuzzleCellValue::Filled)
    }

    /// Create a new letter cell with the given value.
    ///
    /// The value must be an ASCII alphabetic character.
    pub fn valued(val: char) -> Self {
        assert!(val.is_ascii_alphabetic());
        Self {
            no: None,
            val: PuzzleCellValue::Letter(val),
            is_selected: false,
        }
    }

    /// Create a new letter cell with the given value and number.
    ///
    /// The value must be an ASCII alphabetic character.
    /// The number must be less than 100.
    pub fn valnum(val: char, no: u16) -> Self {
        // TODO: are there puzzles with more than 99 clues?
        assert!(no < 100, "Cell number must be less than 100");
        let mut cell = Self::valued(val);
        cell.no = Some(no);
        cell
    }

    pub fn to_val_span(&self) -> Span {
        match &self.val {
            PuzzleCellValue::Empty => Span::raw(BOX_EMPTY.to_string()),
            PuzzleCellValue::Filled => {
                Span::styled(BOX_FILLED.to_string(), Style::default().bg(Color::Black))
            }
            PuzzleCellValue::Letter(c) => Span::raw(c.to_string()),
        }
    }

    pub fn to_no_spans(&self, border_style: Style) -> (Span, Span) {
        match self.no {
            None => (
                Span::styled(BOX_H.to_string(), border_style),
                Span::styled(BOX_H.to_string(), border_style),
            ),
            Some(n) if n < 10 => (
                Span::raw(n.to_string()),
                Span::styled(BOX_H.to_string(), border_style),
            ),
            Some(n) => (
                Span::raw((n / 10).to_string()),
                Span::raw((n % 10).to_string()),
            ),
        }
    }

    pub fn to_selection_span(&self) -> Span {
        if self.is_selected {
            Span::raw("_")
        } else {
            Span::raw(" ")
        }
    }
}

/// A grid of cells.
pub struct PuzzleGrid {
    cells: Vec<Vec<PuzzleCell>>,
}

impl PuzzleGrid {
    pub fn cells(&self) -> &Vec<Vec<PuzzleCell>> {
        &self.cells
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

    /// Convert a [`PuzzleCell`] grid to a [`Paragraph`] for rendering.
    ///
    /// Draws the left & top borders of each cell only once, so that adjacent cells share borders.
    /// Then, the right & bottom borders are drawn only for the last row/column of cells.
    pub fn to_par(&self) -> Paragraph {
        let empty_span = Span::raw(" "); // to be re-used

        let num_rows = self.cells.len();
        let num_cols = self.cells[0].len();
        let mut all_lines: Vec<Line> = Vec::new();

        for (row_idx, cell_line) in self.cells.iter().enumerate() {
            let is_last_row = row_idx == num_rows - 1;

            // Each row of cells produces either 4 or 5 lines:
            // - Always: top border + 3 content lines
            // - Only for last row: bottom border line
            let num_lines = if is_last_row { 5 } else { 4 };
            let mut span_groups: Vec<Vec<Span>> = vec![Vec::new(); num_lines];

            for (col_idx, cell) in cell_line.iter().enumerate() {
                let is_last_col = col_idx == num_cols - 1;

                let border_style = if cell.is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(ratatui::style::Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                // cell-spans
                let val_span = cell.to_val_span();
                let selection_span = cell.to_selection_span();
                let (no_span_1, no_span_2) = cell.to_no_spans(border_style);

                // horizontal & vertical line spans
                let h_span = Span::styled(BOX_H.to_string(), border_style);
                let v_span = Span::styled(BOX_V.to_string(), border_style);

                // top-left corner logic
                let tl_span = if row_idx == 0 && col_idx == 0 {
                    Span::styled(BOX_TL.to_string(), border_style)
                } else if row_idx == 0 {
                    Span::styled(BOX_T.to_string(), border_style)
                } else if col_idx == 0 {
                    Span::styled(BOX_L.to_string(), border_style)
                } else {
                    Span::styled(BOX_X.to_string(), border_style)
                };

                // top line: always draw left border + top border + content
                #[rustfmt::skip]
                span_groups[0].extend([
                    tl_span,
                    no_span_1,
                    no_span_2,
                    h_span.clone(),
                ]);

                // add top-right corner only for last column
                if is_last_col {
                    let tr_span = if row_idx == 0 {
                        Span::styled(BOX_TR.to_string(), border_style)
                    } else {
                        Span::styled(BOX_R.to_string(), border_style)
                    };
                    span_groups[0].push(tr_span);
                }

                // content lines (3 lines): always draw left border + content
                #[rustfmt::skip]
                span_groups[1].extend([
                    v_span.clone(),
                    empty_span.clone(),
                    empty_span.clone(),
                    empty_span.clone(),
                ]);

                #[rustfmt::skip]
                span_groups[2].extend([
                    v_span.clone(),
                    empty_span.clone(),
                    val_span,
                    empty_span.clone(),
                ]);

                #[rustfmt::skip]
                span_groups[3].extend([
                    v_span.clone(),
                    empty_span.clone(),
                    selection_span,
                    empty_span.clone(),
                ]);

                // add right border only for last column
                if is_last_col {
                    span_groups[1].push(v_span.clone());
                    span_groups[2].push(v_span.clone());
                    span_groups[3].push(v_span.clone());
                }

                // bottom line: only for last row
                if is_last_row {
                    let bl_span = if col_idx == 0 {
                        Span::styled(BOX_BL.to_string(), border_style)
                    } else {
                        Span::styled(BOX_B.to_string(), border_style)
                    };

                    #[rustfmt::skip]
                    span_groups[4].extend([
                        bl_span,
                        h_span.clone(),
                        h_span.clone(),
                        h_span.clone(),
                    ]);

                    // add bottom-right corner only for last column
                    if is_last_col {
                        span_groups[4].push(Span::styled(BOX_BR.to_string(), border_style));
                    }
                }
            }

            // Convert span vecs to lines and add to all_lines
            for span_group in span_groups {
                all_lines.push(Line::from_iter(span_group));
            }
        }

        Paragraph::new(all_lines)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::widgets::Widget;

    use super::*;

    #[test]
    fn test_puzzle_cell_to_par() {
        type PC = PuzzleCell;

        #[rustfmt::skip]
        let cells = vec![
            vec![PC::empty(),        PC::filled(),    PC::valnum('B', 12)],
            vec![PC::valnum('A', 1), PC::valued('C'), PC::valued('E')],
            vec![PC::filled(),       PC::filled(),    PC::empty()],
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
