use crate::{Grid, Clue, Direction, Cell};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crossword {
    pub title: String,
    pub author: String,
    pub grid: Grid,
    pub clues: Vec<Clue>,
}

impl Crossword {
    pub fn new(title: String, author: String, width: usize, height: usize) -> Self {
        Self {
            title,
            author,
            grid: Grid::new(width, height),
            clues: Vec::new(),
        }
    }

    pub fn add_clue(&mut self, clue: Clue) -> Result<(), String> {
        self.validate_clue(&clue)?;
        
        for (i, (row, col)) in clue.positions().iter().enumerate() {
            let correct_char = clue.answer.chars().nth(i).unwrap();
            let number = if i == 0 { Some(clue.number) } else { None };
            
            match self.grid.get_cell_mut(*row, *col) {
                Some(cell) => {
                    match cell {
                        Cell::Empty => {
                            *cell = Cell::Letter {
                                correct: correct_char,
                                user_input: None,
                                number,
                            };
                        }
                        Cell::Letter { correct, number: cell_number, .. } => {
                            if *correct != correct_char {
                                return Err(format!("Letter conflict at ({}, {})", row, col));
                            }
                            if i == 0 && cell_number.is_none() {
                                *cell_number = Some(clue.number);
                            }
                        }
                        Cell::Blocked => {
                            return Err(format!("Cannot place letter on blocked cell at ({}, {})", row, col));
                        }
                    }
                }
                None => {
                    return Err(format!("Position ({}, {}) is out of bounds", row, col));
                }
            }
        }
        
        self.clues.push(clue);
        Ok(())
    }

    fn validate_clue(&self, clue: &Clue) -> Result<(), String> {
        if clue.answer.is_empty() {
            return Err("Clue answer cannot be empty".to_string());
        }
        
        let positions = clue.positions();
        if positions.iter().any(|(r, c)| *r >= self.grid.height || *c >= self.grid.width) {
            return Err("Clue extends beyond grid bounds".to_string());
        }
        
        Ok(())
    }

    pub fn get_clues_by_direction(&self, direction: Direction) -> Vec<&Clue> {
        self.clues
            .iter()
            .filter(|clue| clue.direction == direction)
            .collect()
    }

    pub fn get_clue_at_position(&self, row: usize, col: usize, direction: Direction) -> Option<&Clue> {
        self.clues
            .iter()
            .find(|clue| {
                clue.direction == direction && 
                clue.positions().contains(&(row, col))
            })
    }

    pub fn set_blocked_cell(&mut self, row: usize, col: usize) -> Result<(), String> {
        self.grid.set_cell(row, col, Cell::Blocked)
    }

    pub fn validate_solution(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        for clue in &self.clues {
            for (_i, (row, col)) in clue.positions().iter().enumerate() {
                if let Some(Cell::Letter { correct, user_input, .. }) = self.grid.get_cell(*row, *col) {
                    if let Some(input) = user_input {
                        if *input != *correct {
                            errors.push(format!(
                                "Incorrect letter at ({}, {}) for clue {}{}", 
                                row, col, clue.number,
                                match clue.direction {
                                    Direction::Across => "A",
                                    Direction::Down => "D",
                                }
                            ));
                        }
                    }
                }
            }
        }
        
        errors
    }

    pub fn is_complete(&self) -> bool {
        self.grid.is_complete()
    }
}