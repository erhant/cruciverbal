# Cruciverbal

## Project Overview

Cruciverbal is a terminal-based crossword puzzle player written in Rust. The application provides a TUI (Terminal User Interface) for solving crossword puzzles from various providers. Currently, it supports downloading and displaying puzzles from Lovatt's Cryptic crossword service.

**Key Features:**

- Terminal-based crossword puzzle interface
- Async puzzle downloading from external providers
- Grid rendering with scrollable viewport
- PUZ format puzzle parsing and display
- Interactive puzzle solving with keyboard navigation
- Auto-scrolling to keep selected cell visible
- Direction toggling (Across/Down) with word highlighting
- 3-area game layout: top bar (date, title, timer), grid, bottom bar (clue)
- Puzzle selection screen with date picker and provider selector

**Tech Stack:**

- Language: Rust (Edition 2024)
- TUI Framework: Ratatui v0.29 with Crossterm
- Async Runtime: Tokio
- Puzzle Format: PUZ format via `puz-parse` crate
- HTTP Client: Reqwest

## Architecture & Structure

The project is organized as a Cargo workspace with two main crates:

### 1. `tui/` - Terminal User Interface

The main application binary that provides the interactive crossword solving experience.

**Structure:**

- `main.rs` - Application entry point
- `app.rs` - Core application loop, event handling, and view management
- `views/` - View modules for different screens
  - `menu.rs` - Main menu with navigation
  - `game/` - Game view components
    - `mod.rs` - Game state, input handling, and navigation logic
    - `grid.rs` - PuzzleGrid struct with rendering and selection management
    - `cell.rs` - PuzzleCell, PuzzleCellValue, ClueNoDirection, WordIdxDirection
    - `boxchars.rs` - Unicode box-drawing character constants
- `components/` - Reusable UI components (currently empty, for future use)

**Key Components:**

- `App` - Main application state machine with view switching
- `AppView` enum - Menu vs Game view states
- `GameView` enum - Playing, Selecting, Loading, Saving states
- `GameState` - Game state including:
  - `puzzle: Option<Puzzle>` - Raw puzzle data from puz_parse
  - `grid: Option<PuzzleGrid>` - Playable grid built from puzzle
  - `sel: (usize, usize)` - Currently selected cell (row, col)
  - `active_direction: Direction` - Current direction for clue display and word highlighting
  - `puzzle_date: Option<String>` - Date of the loaded puzzle
  - `start_time: Option<Instant>` - Timer start time
  - `selection: SelectionState` - State for puzzle selection screen
  - `visible_area`, `scroll_cur`, `scroll_max`, `scroll_bar` - Viewport/scrolling state
- `SelectionState` - State for puzzle selection:
  - `date: String` - Date input (YYYY-MM-DD)
  - `provider_idx: usize` - Selected provider index
  - `active_field: SelectionField` - Which field is active (Date/Provider/Start)
  - `error: Option<String>` - Error message to display
- `Provider` enum - Available puzzle providers (currently: LovattsCryptic)
- `Direction` enum - Across or Down direction
- `MenuState` - Menu selection tracking
- `PuzzleGrid` - Grid with cells, selection management, and rendering:
  - `from_solution(&[String])` - Build grid from puz_parse solution with automatic clue numbering
  - `set_selection(row, col, direction)` - Cell and word selection with highlighting
  - `get()`, `get_mut()` - Cell access
  - `to_par()` - Convert to Ratatui Paragraph for rendering
- `PuzzleCell` - Individual cell with:
  - `val: PuzzleCellValue` - Either `Filled` (black) or `Letter` with clue data
  - `is_selected_cell: bool` - Whether this is the cursor cell (yellow)
  - `is_selected_word: bool` - Whether this cell is part of the selected word (cyan)
  - `set_user_letter()`, `get_user_letter()` - User input management
  - `clue_no_for_direction()`, `word_idx_for_direction()`, `has_direction()` - Direction helpers
- `ClueNoDirection` - Tracks which clue number(s) the cell belongs to (Across/Down/Cross)
- `WordIdxDirection` - Tracks 0-based position within the word(s)

### 2. `providers/` - Puzzle Providers Library

Library crate for fetching puzzles from external sources.

**Structure:**

- `lib.rs` - Module exports
- `errors.rs` - Provider error types
- `providers/` - Individual provider implementations
  - `lovatts_cryptic.rs` - Lovatt's Cryptic crossword provider

**Key Components:**

- `ProviderError` - Unified error type for fetch/parse failures
- Provider modules implement `download()` functions returning `Result<Puzzle, ProviderError>`

## Key Components

### Application Flow (tui/src/app.rs)

1. Initialize terminal and app state
2. Run event loop at 35 FPS
3. Handle crossterm events (keyboard input)
4. Render current view (Menu, Game:Selecting, Game:Loading, Game:Playing)
5. When in Loading state, download puzzle from selected provider
6. On successful download, transition to Playing state

### Rendering System (tui/src/views/game/grid.rs)

- Custom grid rendering using Unicode box-drawing characters (double-line style: ╔╗╚╝╬ etc.)
- Each cell is 4 lines tall × 4 chars wide (borders shared between adjacent cells)
- Supports:
  - Cell numbering (1-2 digit clue numbers displayed in top-left)
  - Letter display (user-entered letters shown in cell center)
  - Selection indicator (underscore when selected)
  - Yellow highlighting for cursor cell borders
  - Cyan highlighting for word cells in active direction
- Scrollbars (vertical and horizontal) when puzzle exceeds viewport
- Auto-scroll keeps selected cell visible during navigation

### Game Layout (tui/src/views/game/mod.rs)

The game view is split into 3 areas:
- **Top bar**: Date (left), puzzle title (center), timer MM:SS (right)
- **Middle**: Puzzle grid with scrollbars
- **Bottom bar**: Current clue based on selection and active direction

### Puzzle Selection Screen

Centered form with:
- Date input field (YYYY-MM-DD, defaults to today)
- Provider selector (currently only Lovatts Cryptic)
- Start button
- Error message display for validation/download failures

### Puzzle Provider (providers/src/providers/lovatts_cryptic.rs)

- Fetches puzzle data from Puzzleexperts API
- Parses custom `data` field format (ampersand-separated key-value pairs)
- Constructs PUZ-format puzzle with:
  - Grid construction from clue words and positions
  - Automatic clue numbering based on grid positions
  - Separation of across/down clues

### Input Handling

- **Global:** CTRL+C to quit
- **Menu:** Arrow keys for navigation, Enter to select, ESC to quit
- **Puzzle Selection:**
  - ↑↓: Navigate between fields (Date, Provider, Start)
  - ←→: Change provider selection
  - Tab: Next field
  - Enter: Confirm/Start game
  - Type digits/dashes: Edit date field
  - Backspace: Delete character in date field
  - ESC: Back to menu
- **Game (Playing):**
  - Arrow keys: Navigate between cells (skips filled/black cells)
  - A-Z: Enter letter in current cell (auto-uppercased)
  - Backspace/Delete: Clear current cell
  - SPACEBAR: Toggle direction (Across ↔ Down)
  - ESC: Back to menu

## Development Notes

### Current State

- **Fully playable** crossword puzzle experience
- Menu system with New Game option
- Puzzle selection screen with date picker and provider selector
- Async puzzle download with error handling
- Grid rendering with proper border handling
- Cell and word selection with direction toggling
- Yellow highlighting for cursor, cyan for word
- Clue display in bottom bar based on active direction
- Timer display in top bar
- Auto-scroll keeps selection visible
- Scrollbars for large puzzles

### Known Limitations

- Only one provider implemented (Lovatt's Cryptic)
- No rebus square support (noted in tui/src/lib.rs:15)
- No answer validation/checking yet
- No save/load game progress
- Error handling incomplete in some areas (todo!() macros present)
- GameView::Saving not implemented

### Planned Features (from code TODOs)

From tui/src/lib.rs:7-15:

- Save progress locally (CTRL+S)
- Toggle timer (CTRL+T)
- Reveal answers (CTRL+R)
- Improved quit handling (CTRL+Q)

### Configuration

- Workspace uses Rust Edition 2024
- Clippy lint: `uninlined_format_args` allowed
- All workspace members share common dependency versions

### Dependencies Overview

**Workspace-level:**

- `serde` with derive feature for serialization
- `serde_json` for JSON parsing
- `thiserror` for error handling
- `puz-parse` for PUZ format support (with json/serde features)
- `tokio` with full feature set for async runtime
- `reqwest` with json feature for HTTP requests
- `chrono` for date handling

**TUI-specific:**

- `ratatui` v0.29 with unstable-rendered-line-info feature
- `crossterm` v0.29 with event-stream feature
- `color-eyre` v0.6.3 for error reporting
- `futures` v0.3.31 for async utilities
- `chrono` v0.4 for date handling (default date, date validation)

### Development Setup

1. Requires Rust toolchain with 2024 edition support
2. Run with: `cargo run -p cruciverbal-tui`
3. Build providers separately: `cargo build -p cruciverbal-providers`
4. Example puzzle download: See `providers/examples/sample_puzzle.rs`

## Testing Strategy

### Current Tests

- `providers/src/providers/lovatts_cryptic.rs:227` - Download test (async)
- `tui/src/views/game/grid.rs` - Grid tests:
  - `test_from_solution` - Verifies clue numbering and word index assignment
  - `test_puzzle_cell_to_par` - Grid rendering test with buffer inspection

### Test Approach

- Unit tests for parsing logic
- Integration tests for provider downloads
- Visual buffer tests for rendering verification

### Running Tests

```bash
# All tests
cargo test

# Provider tests only
cargo test -p cruciverbal-providers

# TUI tests only
cargo test -p cruciverbal-tui
```

## References

The project draws inspiration from:

- [xword-dl](https://github.com/thisisparker/xword-dl) - Crossword downloader
- [cursewords](https://github.com/thisisparker/cursewords) - Terminal crossword player
- [cliptic](https://github.com/apexatoll/cliptic) - CLI crossword app
- [Daily Crossword Links](https://dailycrosswordlinks.com/)
- [Crossword Fiend](https://crosswordfiend.com/download/)
