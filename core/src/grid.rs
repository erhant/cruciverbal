use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Blocked,
    Letter { 
        correct: char, 
        user_input: Option<char>,
        number: Option<u32>,
    },
}

impl Cell {
    pub fn is_blocked(&self) -> bool {
        matches!(self, Cell::Blocked)
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }

    pub fn is_letter(&self) -> bool {
        matches!(self, Cell::Letter { .. })
    }

    pub fn get_display_char(&self) -> char {
        match self {
            Cell::Empty => ' ',
            Cell::Blocked => '█',
            Cell::Letter { user_input: Some(c), .. } => *c,
            Cell::Letter { user_input: None, .. } => ' ',
        }
    }

    pub fn get_number(&self) -> Option<u32> {
        match self {
            Cell::Letter { number, .. } => *number,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Cell>>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![Cell::Empty; width]; height];
        Self {
            width,
            height,
            cells,
        }
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&Cell> {
        self.cells.get(row)?.get(col)
    }

    pub fn get_cell_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        self.cells.get_mut(row)?.get_mut(col)
    }

    pub fn set_cell(&mut self, row: usize, col: usize, cell: Cell) -> Result<(), String> {
        if row >= self.height || col >= self.width {
            return Err(format!("Position ({}, {}) is out of bounds", row, col));
        }
        self.cells[row][col] = cell;
        Ok(())
    }

    pub fn set_user_input(&mut self, row: usize, col: usize, c: char) -> Result<(), String> {
        match self.get_cell_mut(row, col) {
            Some(Cell::Letter { user_input, .. }) => {
                *user_input = Some(c.to_ascii_uppercase());
                Ok(())
            }
            Some(_) => Err("Cannot input letter in non-letter cell".to_string()),
            None => Err("Position out of bounds".to_string()),
        }
    }

    pub fn clear_user_input(&mut self, row: usize, col: usize) -> Result<(), String> {
        match self.get_cell_mut(row, col) {
            Some(Cell::Letter { user_input, .. }) => {
                *user_input = None;
                Ok(())
            }
            Some(_) => Err("Cannot clear non-letter cell".to_string()),
            None => Err("Position out of bounds".to_string()),
        }
    }

    pub fn is_complete(&self) -> bool {
        for row in &self.cells {
            for cell in row {
                if let Cell::Letter { correct, user_input, .. } = cell {
                    if user_input.is_none() || user_input != &Some(*correct) {
                        return false;
                    }
                }
            }
        }
        true
    }
}