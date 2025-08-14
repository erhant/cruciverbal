use crate::Direction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clue {
    pub number: u32,
    pub direction: Direction,
    pub text: String,
    pub answer: String,
    pub row: usize,
    pub col: usize,
}

impl Clue {
    pub fn new(number: u32, direction: Direction, text: String, answer: String, row: usize, col: usize) -> Self {
        Self {
            number,
            direction,
            text,
            answer: answer.to_uppercase(),
            row,
            col,
        }
    }

    pub fn length(&self) -> usize {
        self.answer.len()
    }

    pub fn positions(&self) -> Vec<(usize, usize)> {
        let (dr, dc) = self.direction.delta();
        (0..self.length())
            .map(|i| {
                (
                    (self.row as i32 + dr * i as i32) as usize,
                    (self.col as i32 + dc * i as i32) as usize,
                )
            })
            .collect()
    }
}