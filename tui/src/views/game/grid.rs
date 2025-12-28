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
pub struct PuzzleCell {
    /// Number of the clue starting at this cell, if any.
    pub no: Option<u16>,
    /// Value contained within this cell.
    pub val: PuzzleCellValue,
    /// Whether this cell is currently selected by the user.
    pub is_selected: bool,
}

pub enum PuzzleCellValue {
    Empty,        // empty cell
    Filled,       // black cell
    Letter(char), // letter cell
}

impl PuzzleCell {
    /// Create a new empty cell.
    pub fn new_empty() -> Self {
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
    pub fn new_filled() -> Self {
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
    pub fn new_valued(val: char) -> Self {
        assert!(val.is_ascii_alphabetic());
        Self {
            no: None,
            val: PuzzleCellValue::Letter(val),
            is_selected: false,
        }
    }

    pub fn to_val_span(&self) -> Span {
        match &self.val {
            PuzzleCellValue::Empty => Span::raw(" "),
            PuzzleCellValue::Filled => Span::styled(" ", Style::default().bg(Color::Black)),
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

    /// Convert a [`PuzzleCell`] grid to a [`Paragraph`] for rendering.
    pub fn to_par(&self) -> Paragraph {
        let empty_span = Span::raw(" "); // to be re-used

        let mut line_groups: Vec<[Line; 5]> = Vec::with_capacity(self.cells.len());
        for (row_idx, cell_line) in self.cells.iter().enumerate() {
            // each line of cells produces 5 lines of `5 x N` spans each (to form the box)
            let mut span_groups: [Vec<Span>; 5] = [
                Vec::with_capacity(cell_line.len() * 5),
                Vec::with_capacity(cell_line.len() * 5),
                Vec::with_capacity(cell_line.len() * 5),
                Vec::with_capacity(cell_line.len() * 5),
                Vec::with_capacity(cell_line.len() * 5),
            ];
            for (col_idx, cell) in cell_line.iter().enumerate() {
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
                let tl_span = if row_idx == 0 {
                    Span::styled(BOX_TL.to_string(), border_style)
                } else if col_idx == 0 {
                    Span::styled(BOX_L.to_string(), border_style)
                } else {
                    Span::styled(BOX_X.to_string(), border_style)
                };

                // top-right corner logic
                let tr_span = if row_idx == 0 {
                    Span::styled(BOX_TR.to_string(), border_style)
                } else if col_idx == self.cells[0].len() - 1 {
                    Span::styled(BOX_R.to_string(), border_style)
                } else {
                    Span::styled(BOX_X.to_string(), border_style)
                };

                // bottom-left corner logic
                let bl_span = if row_idx == self.cells.len() - 1 {
                    Span::styled(BOX_BL.to_string(), border_style)
                } else if col_idx == 0 {
                    Span::styled(BOX_L.to_string(), border_style)
                } else {
                    Span::styled(BOX_X.to_string(), border_style)
                };

                // bottom-right corner logic
                let br_span = if row_idx == self.cells.len() - 1 {
                    Span::styled(BOX_BR.to_string(), border_style)
                } else if col_idx == self.cells[0].len() - 1 {
                    Span::styled(BOX_R.to_string(), border_style)
                } else {
                    Span::styled(BOX_X.to_string(), border_style)
                };

                // top line
                #[rustfmt::skip]
                span_groups[0].extend([
                  tl_span,
                  no_span_1,
                  no_span_2,
                  h_span.clone(),
                  tr_span
                ]);
                // second line
                span_groups[1].extend([
                    v_span.clone(),
                    empty_span.clone(),
                    empty_span.clone(),
                    empty_span.clone(),
                    v_span.clone(),
                ]);
                // middle line
                span_groups[2].extend([
                    v_span.clone(),
                    empty_span.clone(),
                    val_span,
                    empty_span.clone(),
                    v_span.clone(),
                ]);
                // fourth line
                span_groups[3].extend([
                    v_span.clone(),
                    empty_span.clone(),
                    selection_span,
                    empty_span.clone(),
                    v_span.clone(),
                ]);
                // bottom line
                span_groups[4].extend([
                    bl_span,
                    h_span.clone(),
                    h_span.clone(),
                    h_span.clone(),
                    br_span,
                ]);
            }

            // now convert span vecs to lines
            let lines = span_groups.map(Line::from_iter);
            line_groups.push(lines);
        }

        // flatten all lines & turn them into a paragraph
        let all_lines: Vec<Line> = line_groups
            .into_iter()
            .map(|l| l.to_vec())
            .flatten()
            .collect();
        let paragraph = Paragraph::new(all_lines);

        paragraph
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use ratatui::widgets::Widget;

    use super::*;

    #[test]
    fn test_puzzle_cell_to_par() {
        let cells = vec![
            vec![PuzzleCell::new_empty(), PuzzleCell::new_filled()],
            vec![PuzzleCell::new_valued('A'), PuzzleCell::new_filled()],
        ];

        let grid = PuzzleGrid::new(cells);
        let par = grid.to_par();

        // create a dummy area for rendering
        let (width, height) = (50, 20);
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
