use crate::{Crossword, Grid, Cell, Clue, Direction};
use std::collections::{HashMap, HashSet};

/// A word entry with its clue for puzzle generation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordEntry {
    pub word: String,
    pub clue: String,
}

impl WordEntry {
    pub fn new(word: String, clue: String) -> Self {
        Self {
            word: word.to_uppercase(),
            clue,
        }
    }
    
    pub fn len(&self) -> usize {
        self.word.len()
    }
}

/// Represents a potential word placement in the grid
#[derive(Debug, Clone)]
struct Placement {
    word: String,
    clue: String,
    row: usize,
    col: usize,
    direction: Direction,
    intersections: Vec<Intersection>,
}

/// Represents where two words intersect
#[derive(Debug, Clone)]
struct Intersection {
    row: usize,
    col: usize,
    letter: char,
    word1_index: usize, // Index in the first word
    word2_index: usize, // Index in the second word
}

/// Generator configuration parameters
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub width: usize,
    pub height: usize,
    pub max_words: usize,
    pub min_words: usize,
    pub symmetry: bool,
    pub max_attempts: usize,
    pub prefer_longer_words: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            width: 15,
            height: 15,
            max_words: 50,
            min_words: 20,
            symmetry: true,
            max_attempts: 100,
            prefer_longer_words: true,
        }
    }
}

/// Quality metrics for generated puzzles
#[derive(Debug, Clone)]
pub struct PuzzleQuality {
    pub word_count: usize,
    pub intersection_count: usize,
    pub average_word_length: f32,
    pub grid_fill_percentage: f32,
    pub symmetry_score: f32,
    pub total_score: f32,
}

/// The main crossword puzzle generator
pub struct CrosswordGenerator {
    config: GeneratorConfig,
    word_list: Vec<WordEntry>,
    grid: Grid,
    placed_words: Vec<Placement>,
    blocked_positions: HashSet<(usize, usize)>,
    word_starts: HashMap<(usize, usize), u32>, // Position -> clue number
    next_clue_number: u32,
}

impl CrosswordGenerator {
    /// Create a new generator with the given configuration
    pub fn new(config: GeneratorConfig) -> Self {
        let grid = Grid::new(config.width, config.height);
        Self {
            config,
            word_list: Vec::new(),
            grid,
            placed_words: Vec::new(),
            blocked_positions: HashSet::new(),
            word_starts: HashMap::new(),
            next_clue_number: 1,
        }
    }

    /// Add words to the generator's vocabulary
    pub fn add_words(&mut self, words: Vec<(String, String)>) {
        for (word, clue) in words {
            if word.len() >= 3 && word.chars().all(|c| c.is_alphabetic()) {
                self.word_list.push(WordEntry::new(word, clue));
            }
        }
        
        // Sort words by length (longer words first if preferred)
        if self.config.prefer_longer_words {
            self.word_list.sort_by(|a, b| b.len().cmp(&a.len()));
        }
    }

    /// Generate a crossword puzzle
    pub fn generate(&mut self) -> Result<Crossword, GeneratorError> {
        for attempt in 0..self.config.max_attempts {
            self.reset_grid();
            
            // Try to generate a puzzle
            if let Ok(crossword) = self.attempt_generation() {
                let quality = self.evaluate_quality(&crossword);
                
                // Accept if it meets minimum requirements
                if quality.word_count >= self.config.min_words {
                    return Ok(crossword);
                }
            }
            
            // Shuffle word list for next attempt
            if attempt < self.config.max_attempts - 1 {
                self.shuffle_words();
            }
        }
        
        Err(GeneratorError::MaxAttemptsExceeded)
    }

    /// Reset the grid for a new generation attempt
    fn reset_grid(&mut self) {
        self.grid = Grid::new(self.config.width, self.config.height);
        self.placed_words.clear();
        self.blocked_positions.clear();
        self.word_starts.clear();
        self.next_clue_number = 1;
        
        // Generate initial blocked cell pattern
        if self.config.symmetry {
            self.generate_symmetric_pattern();
        } else {
            self.generate_random_pattern();
        }
    }

    /// Generate a symmetric blocked cell pattern
    fn generate_symmetric_pattern(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.config.width.hash(&mut hasher);
        self.config.height.hash(&mut hasher);
        let seed = hasher.finish();
        
        // Simple symmetric pattern generation
        // In a real implementation, you'd want more sophisticated pattern generation
        let center_row = self.config.height / 2;
        let center_col = self.config.width / 2;
        
        // Add some blocked cells with rotational symmetry
        for row in 0..=center_row {
            for col in 0..=center_col {
                // Simple deterministic pattern based on position
                let hash = (row * 31 + col * 17 + seed as usize) % 100;
                if hash < 15 { // ~15% blocked cells
                    self.add_symmetric_blocked_cell(row, col);
                }
            }
        }
    }

    /// Generate a random blocked cell pattern
    fn generate_random_pattern(&mut self) {
        // Simple random pattern - in practice you'd want better algorithms
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.config.width.hash(&mut hasher);
        let seed = hasher.finish() as usize;
        
        for row in 0..self.config.height {
            for col in 0..self.config.width {
                let hash = (row * 31 + col * 17 + seed) % 100;
                if hash < 12 { // ~12% blocked cells
                    self.blocked_positions.insert((row, col));
                    let _ = self.grid.set_cell(row, col, Cell::Blocked);
                }
            }
        }
    }

    /// Add a blocked cell with rotational symmetry
    fn add_symmetric_blocked_cell(&mut self, row: usize, col: usize) {
        let positions = [
            (row, col),
            (self.config.height - 1 - row, self.config.width - 1 - col),
        ];
        
        for &(r, c) in &positions {
            if r < self.config.height && c < self.config.width {
                self.blocked_positions.insert((r, c));
                let _ = self.grid.set_cell(r, c, Cell::Blocked);
            }
        }
    }

    /// Attempt to generate a puzzle with the current configuration
    fn attempt_generation(&mut self) -> Result<Crossword, GeneratorError> {
        // Find potential word slots
        let slots = self.find_word_slots();
        
        // Try to fill slots with words using backtracking
        self.fill_slots_with_backtracking(&slots)?;
        
        // Build the final crossword
        self.build_crossword()
    }

    /// Find all potential slots where words can be placed
    fn find_word_slots(&self) -> Vec<WordSlot> {
        let mut slots = Vec::new();
        
        // Find horizontal slots
        for row in 0..self.config.height {
            let mut start_col = None;
            
            for col in 0..=self.config.width {
                let is_blocked = col == self.config.width || 
                    self.blocked_positions.contains(&(row, col));
                
                if is_blocked {
                    if let Some(start) = start_col {
                        let length = col - start;
                        if length >= 3 {
                            slots.push(WordSlot {
                                row,
                                col: start,
                                length,
                                direction: Direction::Across,
                            });
                        }
                    }
                    start_col = None;
                } else if start_col.is_none() {
                    start_col = Some(col);
                }
            }
        }
        
        // Find vertical slots
        for col in 0..self.config.width {
            let mut start_row = None;
            
            for row in 0..=self.config.height {
                let is_blocked = row == self.config.height || 
                    self.blocked_positions.contains(&(row, col));
                
                if is_blocked {
                    if let Some(start) = start_row {
                        let length = row - start;
                        if length >= 3 {
                            slots.push(WordSlot {
                                row: start,
                                col,
                                length,
                                direction: Direction::Down,
                            });
                        }
                    }
                    start_row = None;
                } else if start_row.is_none() {
                    start_row = Some(row);
                }
            }
        }
        
        // Sort slots by length (longer first for better constraint propagation)
        slots.sort_by(|a, b| b.length.cmp(&a.length));
        slots
    }

    /// Fill word slots using backtracking algorithm
    fn fill_slots_with_backtracking(&mut self, slots: &[WordSlot]) -> Result<(), GeneratorError> {
        self.backtrack(0, slots)
    }

    /// Recursive backtracking function
    fn backtrack(&mut self, slot_index: usize, slots: &[WordSlot]) -> Result<(), GeneratorError> {
        // Base case: all slots filled successfully
        if slot_index >= slots.len() || self.placed_words.len() >= self.config.max_words {
            return Ok(());
        }
        
        let slot = &slots[slot_index];
        let candidate_words = self.find_candidate_words(slot);
        
        // Try each candidate word
        for word_entry in candidate_words {
            if self.can_place_word(&word_entry, slot) {
                // Try placing the word
                self.place_word(&word_entry, slot);
                
                // Recursively try to fill remaining slots
                if self.backtrack(slot_index + 1, slots).is_ok() {
                    return Ok(()); // Success!
                }
                
                // Backtrack: remove the word
                self.remove_last_word();
            }
        }
        
        // Try skipping this slot (partial fill is acceptable)
        self.backtrack(slot_index + 1, slots)
    }

    /// Build the final crossword from placed words
    fn build_crossword(&mut self) -> Result<Crossword, GeneratorError> {
        let mut crossword = Crossword::new(
            "Generated Puzzle".to_string(),
            "Cruciverbal Generator".to_string(),
            self.config.width,
            self.config.height,
        );
        
        // Copy the grid
        crossword.grid = self.grid.clone();
        
        // Add clues
        for placement in &self.placed_words {
            let clue = Clue::new(
                self.get_clue_number(placement.row, placement.col),
                placement.direction,
                placement.clue.clone(),
                placement.word.clone(),
                placement.row,
                placement.col,
            );
            
            crossword.add_clue(clue)?;
        }
        
        Ok(crossword)
    }

    // Helper methods (implementations would be here)
    fn shuffle_words(&mut self) {
        // Simple shuffle using built-in deterministic ordering
        self.word_list.sort_by(|a, b| a.word.cmp(&b.word));
        self.word_list.reverse();
    }

    fn find_candidate_words(&self, slot: &WordSlot) -> Vec<WordEntry> {
        self.word_list
            .iter()
            .filter(|entry| {
                entry.len() == slot.length &&
                !self.is_word_already_used(&entry.word)
            })
            .cloned()
            .collect()
    }

    fn can_place_word(&self, word_entry: &WordEntry, slot: &WordSlot) -> bool {
        // Check if word can be placed without conflicts
        for (i, letter) in word_entry.word.chars().enumerate() {
            let (row, col) = match slot.direction {
                Direction::Across => (slot.row, slot.col + i),
                Direction::Down => (slot.row + i, slot.col),
            };
            
            if let Some(cell) = self.grid.get_cell(row, col) {
                match cell {
                    Cell::Letter { correct, .. } if *correct != letter => return false,
                    Cell::Blocked => return false,
                    _ => {}
                }
            }
        }
        true
    }

    fn place_word(&mut self, word_entry: &WordEntry, slot: &WordSlot) {
        // Place the word in the grid
        for (i, letter) in word_entry.word.chars().enumerate() {
            let (row, col) = match slot.direction {
                Direction::Across => (slot.row, slot.col + i),
                Direction::Down => (slot.row + i, slot.col),
            };
            
            let number = if i == 0 { Some(self.next_clue_number) } else { None };
            let cell = Cell::Letter {
                correct: letter,
                user_input: None,
                number,
            };
            
            let _ = self.grid.set_cell(row, col, cell);
        }
        
        // Record the placement
        let placement = Placement {
            word: word_entry.word.clone(),
            clue: word_entry.clue.clone(),
            row: slot.row,
            col: slot.col,
            direction: slot.direction,
            intersections: Vec::new(), // TODO: Calculate intersections
        };
        
        self.placed_words.push(placement);
        self.word_starts.insert((slot.row, slot.col), self.next_clue_number);
        self.next_clue_number += 1;
    }

    fn remove_last_word(&mut self) {
        if let Some(placement) = self.placed_words.pop() {
            // Clear the word from the grid
            for i in 0..placement.word.len() {
                let (row, col) = match placement.direction {
                    Direction::Across => (placement.row, placement.col + i),
                    Direction::Down => (placement.row + i, placement.col),
                };
                
                // Only clear if no other words use this position
                if !self.is_position_used_by_other_words(row, col, &placement) {
                    let _ = self.grid.set_cell(row, col, Cell::Empty);
                }
            }
            
            self.word_starts.remove(&(placement.row, placement.col));
            self.next_clue_number -= 1;
        }
    }

    fn is_word_already_used(&self, word: &str) -> bool {
        self.placed_words.iter().any(|p| p.word == word)
    }

    fn is_position_used_by_other_words(&self, row: usize, col: usize, exclude: &Placement) -> bool {
        self.placed_words.iter().any(|p| {
            if p.word == exclude.word { return false; }
            
            for i in 0..p.word.len() {
                let (p_row, p_col) = match p.direction {
                    Direction::Across => (p.row, p.col + i),
                    Direction::Down => (p.row + i, p.col),
                };
                
                if p_row == row && p_col == col {
                    return true;
                }
            }
            false
        })
    }

    fn get_clue_number(&self, row: usize, col: usize) -> u32 {
        *self.word_starts.get(&(row, col)).unwrap_or(&1)
    }

    fn evaluate_quality(&self, crossword: &Crossword) -> PuzzleQuality {
        let word_count = crossword.clues.len();
        let total_letters = crossword.clues.iter().map(|c| c.length()).sum::<usize>();
        let average_word_length = total_letters as f32 / word_count as f32;
        
        let total_cells = (self.config.width * self.config.height) as f32;
        let filled_cells = total_letters as f32;
        let grid_fill_percentage = filled_cells / total_cells * 100.0;
        
        // Simple scoring system
        let total_score = word_count as f32 * 2.0 + 
                         average_word_length * 10.0 + 
                         grid_fill_percentage;
        
        PuzzleQuality {
            word_count,
            intersection_count: 0, // TODO: Calculate actual intersections
            average_word_length,
            grid_fill_percentage,
            symmetry_score: if self.config.symmetry { 100.0 } else { 0.0 },
            total_score,
        }
    }
}

/// Represents a slot where a word can be placed
#[derive(Debug, Clone)]
struct WordSlot {
    row: usize,
    col: usize,
    length: usize,
    direction: Direction,
}

/// Generator error types
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    #[error("Maximum generation attempts exceeded")]
    MaxAttemptsExceeded,
    #[error("Failed to place words: {0}")]
    PlacementFailed(String),
    #[error("Grid error: {0}")]
    GridError(String),
}

impl From<String> for GeneratorError {
    fn from(msg: String) -> Self {
        GeneratorError::PlacementFailed(msg)
    }
}

/// Public API function to generate a crossword
pub fn generate_crossword(
    words: Vec<(String, String)>,
    config: Option<GeneratorConfig>,
) -> Result<Crossword, GeneratorError> {
    let mut generator = CrosswordGenerator::new(config.unwrap_or_default());
    generator.add_words(words);
    generator.generate()
}