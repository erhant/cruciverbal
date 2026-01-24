use super::boxchars::*;
use ratatui::{
    style::{Color, Style},
    text::Span,
};

/// The active direction for navigation and clue display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Across,
    Down,
}

impl Direction {
    /// Toggle between Across and Down.
    pub fn toggle(&self) -> Self {
        match self {
            Direction::Across => Direction::Down,
            Direction::Down => Direction::Across,
        }
    }
}

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
#[derive(Debug)]
pub struct PuzzleCell {
    /// Value contained within this cell.
    pub val: PuzzleCellValue,
    /// Whether this cell is the currently selected cursor cell.
    pub is_selected_cell: bool,
    /// Whether this cell is part of the currently selected word (but not the cursor).
    pub is_selected_word: bool,
}

/// The clue number(s) for the word(s) that pass through this cell.
///
/// In crosswords, each word has a clue number assigned at its starting cell.
/// A cell may belong to an across word, a down word, or both (crossing).
///
/// # Example
///
/// ```txt
/// A C R O S S
/// . E . . . .
/// . L . . . .
/// . L . . . .
/// ```
///
/// - Cell at "A" belongs to across word with clue #1 -> `Across(1)`
/// - Cell at "E" belongs to down word with clue #2 -> `Down(2)` (assuming it starts there)
/// - Cell at "C" belongs to both -> `Cross(1, 2)` (across clue 1, down clue 2)
#[derive(Debug)]
pub enum ClueNoDirection {
    /// Cell belongs only to an across word with this clue number.
    Across(usize),
    /// Cell belongs only to a down word with this clue number.
    Down(usize),
    /// Cell belongs to both an across and down word: `Cross(across_clue, down_clue)`.
    Cross(usize, usize),
}

/// The position of this cell within the word(s) it belongs to.
///
/// Each letter cell is part of one or two words (across, down, or both).
/// This tracks the 0-based index of this cell within each word.
///
/// # Example
///
/// ```txt
/// W O R D
/// ```
///
/// - "W" is at index 0 -> `Across(0)`
/// - "O" is at index 1 -> `Across(1)`
/// - "R" is at index 2 -> `Across(2)`
/// - "D" is at index 3 -> `Across(3)`
///
/// For crossing cells:
///
/// ```txt
/// . C .
/// A B C
/// . C .
/// ```
/// - "B" -> `Cross(1, 1)`
#[derive(Debug)]
pub enum WordIdxDirection {
    /// Cell is only part of an across word, at this index.
    Across(usize),
    /// Cell is only part of a down word, at this index.
    Down(usize),
    /// Cell is part of both words: `Cross(across_idx, down_idx)`.
    Cross(usize, usize),
}

#[derive(Debug)]
pub enum PuzzleCellValue {
    /// An flled (black) cell.
    Filled,
    /// A letter cell.
    Letter {
        /// The clue letter contained in this cell.
        clue_letter: char,
        /// The letter entered by the user, if any.
        user_letter: Option<char>,
        /// Index of this letter in the corresponding across & down words.
        word_idx: WordIdxDirection,
        /// Clue numbers for across & down words starting at this cell, if any.
        clue_no: ClueNoDirection,
    },
}

impl PuzzleCell {
    /// Check if the cell is empty.
    ///
    /// - For letter cells, checks if no letter has been entered by the user.
    /// - For filled cells, always returns `false`.
    pub fn is_empty(&self) -> bool {
        if let PuzzleCellValue::Letter { user_letter, .. } = self.val {
            user_letter.is_none()
        } else {
            false
        }
    }

    /// Create a filled (black) cell.
    pub fn filled() -> Self {
        Self {
            val: PuzzleCellValue::Filled,
            is_selected_cell: false,
            is_selected_word: false,
        }
    }

    /// Create a letter cell with a clue number.
    pub fn valued(clue_letter: char, word_idx: WordIdxDirection, clue_no: ClueNoDirection) -> Self {
        Self {
            val: PuzzleCellValue::Letter {
                clue_letter,
                user_letter: None,
                word_idx,
                clue_no,
            },
            is_selected_cell: false,
            is_selected_word: false,
        }
    }

    pub fn as_selected(mut self) -> Self {
        self.is_selected_cell = true;
        self
    }

    /// Clear all selection flags (cell and word).
    pub fn clear_selection(&mut self) {
        self.is_selected_cell = false;
        self.is_selected_word = false;
    }

    /// Check if the cell is filled (black).
    pub fn is_filled(&self) -> bool {
        matches!(self.val, PuzzleCellValue::Filled)
    }

    /// Set the user-entered letter for this cell.
    ///
    /// Does nothing if the cell is filled.
    pub fn set_user_letter(&mut self, letter: Option<char>) {
        if let PuzzleCellValue::Letter { user_letter, .. } = &mut self.val {
            *user_letter = letter;
        }
    }

    /// Get the user-entered letter, if any.
    pub fn get_user_letter(&self) -> Option<char> {
        if let PuzzleCellValue::Letter { user_letter, .. } = &self.val {
            *user_letter
        } else {
            None
        }
    }

    pub fn to_val_span(&self) -> Span {
        match &self.val {
            PuzzleCellValue::Filled => {
                Span::styled(BOX_FILLED.to_string(), Style::default().bg(Color::Black))
            }
            PuzzleCellValue::Letter { user_letter, .. } => match user_letter {
                Some(c) => Span::raw(c.to_string()),
                None => Span::raw(BOX_EMPTY.to_string()),
            },
        }
    }

    pub fn to_no_spans(&self, border_style: Style) -> (Span, Span) {
        match &self.val {
            // no clue number for filled cells
            PuzzleCellValue::Filled => (
                Span::styled(BOX_H.to_string(), border_style),
                Span::styled(BOX_H.to_string(), border_style),
            ),
            PuzzleCellValue::Letter {
                clue_no, word_idx, ..
            } => {
                let no = match (clue_no, word_idx) {
                    // if across & its the first letter of the word
                    (
                        ClueNoDirection::Across(n),
                        WordIdxDirection::Across(0) | WordIdxDirection::Cross(0, _),
                    ) => Some(*n),
                    // if down & its the first letter of the word
                    (
                        ClueNoDirection::Down(n),
                        WordIdxDirection::Down(0) | WordIdxDirection::Cross(_, 0),
                    ) => Some(*n),
                    // if cross & its the first letter of BOTH words
                    (ClueNoDirection::Cross(a, d), WordIdxDirection::Cross(0, 0)) => {
                        assert!(a == d);
                        Some(*a)
                    }
                    // if cross & its the first letter of either words
                    (ClueNoDirection::Cross(a, _), WordIdxDirection::Cross(0, _)) => Some(*a),
                    (ClueNoDirection::Cross(_, d), WordIdxDirection::Cross(_, 0)) => Some(*d),
                    _ => None,
                };

                match no {
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
        }
    }

    pub fn to_selection_span(&self) -> Span {
        if self.is_selected_cell {
            Span::raw("_")
        } else {
            Span::raw(" ")
        }
    }

    /// Get the clue number for a given direction, if the cell belongs to a word in that direction.
    pub fn clue_no_for_direction(&self, direction: Direction) -> Option<usize> {
        match &self.val {
            PuzzleCellValue::Filled => None,
            PuzzleCellValue::Letter { clue_no, .. } => match (clue_no, direction) {
                (ClueNoDirection::Across(n), Direction::Across) => Some(*n),
                (ClueNoDirection::Down(n), Direction::Down) => Some(*n),
                (ClueNoDirection::Cross(a, _), Direction::Across) => Some(*a),
                (ClueNoDirection::Cross(_, d), Direction::Down) => Some(*d),
                _ => None,
            },
        }
    }

    /// Get the word index for a given direction, if the cell belongs to a word in that direction.
    pub fn word_idx_for_direction(&self, direction: Direction) -> Option<usize> {
        match &self.val {
            PuzzleCellValue::Filled => None,
            PuzzleCellValue::Letter { word_idx, .. } => match (word_idx, direction) {
                (WordIdxDirection::Across(i), Direction::Across) => Some(*i),
                (WordIdxDirection::Down(i), Direction::Down) => Some(*i),
                (WordIdxDirection::Cross(a, _), Direction::Across) => Some(*a),
                (WordIdxDirection::Cross(_, d), Direction::Down) => Some(*d),
                _ => None,
            },
        }
    }

    /// Check if this cell belongs to a word in the given direction.
    pub fn has_direction(&self, direction: Direction) -> bool {
        match &self.val {
            PuzzleCellValue::Filled => false,
            PuzzleCellValue::Letter { clue_no, .. } => match (clue_no, direction) {
                (ClueNoDirection::Across(_), Direction::Across) => true,
                (ClueNoDirection::Down(_), Direction::Down) => true,
                (ClueNoDirection::Cross(_, _), _) => true,
                _ => false,
            },
        }
    }
}
