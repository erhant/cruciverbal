use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Across,
    Down,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Across => (0, 1),
            Direction::Down => (1, 0),
        }
    }
}