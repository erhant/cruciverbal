# Cruciverbal

A Rust-based crossword puzzle game and editor for the terminal.

## Features

- **Play Mode**: Solve crossword puzzles in the terminal
- **Edit Mode**: Create and edit crossword puzzles
- **Puzzle Generation**: Automatically generate random crosswords from word lists
- **File Support**: Import/export puzzles and word lists
- **Terminal Interface**: Powered by Ratatui with 3x3 cell rendering for clear grid display

## Project Structure

This project follows a monorepo approach with three main crates:

### Core (`core/`)

The foundational game engine containing:

- Grid management and cell types
- Clue validation and positioning
- Crossword puzzle logic
- Solution validation

### TUI (`tui/`)

Terminal user interface using Ratatui:

- Interactive puzzle display
- Play mode for solving puzzles
- Edit mode for creating puzzles
- Keyboard navigation and input

### External (`external/`)

File I/O and data management:

- Word list import/export (CSV, JSON)
- Crossword puzzle file formats
- Game state persistence

## Getting Started

### Prerequisites

- Rust 1.70+ (2021 edition)

### Building

```bash
cargo build --release
```

### Running

```bash
cargo run --bin cruciverbal-tui
```

## Controls

### Main Menu

- `↑↓` - Navigate menu options
- `Enter` - Select option (Load Game, New Random Game, Create Puzzle)
- `Q` - Quit

### Load Game

- Type file path and press `Enter` to load
- `Esc` - Cancel and return to menu

### Play Mode

- `↑↓←→` - Move cursor
- `A-Z` - Enter letters
- `Backspace` - Delete and move back
- `Delete` - Delete current letter
- `Tab` - Toggle direction (Across/Down)
- `Enter` - Jump to next empty cell
- `Esc` - Return to main menu

### Edit Mode (Create Puzzle)

- `↑↓←→` - Move cursor
- `Space` - Toggle cell type (Empty/Blocked/Letter)
- `C` - Add clue at current position
- `Tab` - Switch between Grid/Clue edit modes
- `Esc` - Return to main menu

## File Formats

### Word Lists

- **CSV**: `word,clue,difficulty,category`
- **JSON**: Structured format with metadata

### Crossword Files

- **JSON**: Human-readable puzzle format
- **CWB**: Binary format for compact storage

## Examples

Create a new word list:

```rust
use cruciverbal_external::{WordList, WordEntry};

let mut word_list = WordList::new("Animals".to_string());
word_list.add_entry(WordEntry {
    word: "DOG".to_string(),
    clue: "Man's best friend".to_string(),
    difficulty: Some(2),
    category: Some("Animals".to_string()),
});
```

Generate a crossword puzzle:

```rust
use cruciverbal_core::{generate_crossword, GeneratorConfig};

let words = vec![
    ("DOG".to_string(), "Man's best friend".to_string()),
    ("CAT".to_string(), "Feline pet".to_string()),
    ("RUN".to_string(), "Move quickly".to_string()),
];

let config = GeneratorConfig {
    width: 10,
    height: 10,
    max_words: 15,
    symmetry: true,
    ..Default::default()
};

let crossword = generate_crossword(words, Some(config))?;
```

## Development

Each crate can be developed independently:

```bash
# Test core logic
cargo test -p cruciverbal-core

# Test file operations
cargo test -p cruciverbal-external

# Run the TUI
cargo run -p cruciverbal-tui
```

## See Also

- [Blog Post by Science Friday](https://www.sciencefriday.com/articles/inside-the-box-crossword-puzzle-constructing-in-the-computer-age/)
- [Cliptic](https://github.com/apexatoll/cliptic)

## License

MIT License - see LICENSE file for details.
