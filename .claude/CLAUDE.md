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
- `GameState` - Game state including:
  - `puzzle: Option<Puzzle>` - Raw puzzle data from puz_parse
  - `grid: Option<PuzzleGrid>` - Playable grid built from puzzle
  - `sel: (usize, usize)` - Currently selected cell (row, col)
  - `visible_area`, `scroll_cur`, `scroll_max`, `scroll_bar` - Viewport/scrolling state
- `MenuState` - Menu selection tracking
- `PuzzleGrid` - Grid with cells, selection management, and rendering:
  - `from_solution(&[String])` - Build grid from puz_parse solution with automatic clue numbering
  - `set_selection()`, `get()`, `get_mut()` - Cell access and selection management
  - `to_par()` - Convert to Ratatui Paragraph for rendering
- `PuzzleCell` - Individual cell with:
  - `val: PuzzleCellValue` - Either `Filled` (black) or `Letter` with clue data
  - `is_selected: bool` - Whether cell is currently selected
  - `set_user_letter()`, `get_user_letter()` - User input management
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

### Application Flow (tui/src/app.rs:52-84)

1. Initialize terminal and app state
2. Run event loop at 35 FPS
3. Handle crossterm events (keyboard input)
4. Render current view (menu or game)
5. Auto-download puzzle if none loaded (hardcoded date currently)

### Rendering System (tui/src/views/game/grid.rs)

- Custom grid rendering using Unicode box-drawing characters (double-line style: ╔╗╚╝╬ etc.)
- Each cell is 4 lines tall × 4 chars wide (borders shared between adjacent cells)
- Supports:
  - Cell numbering (1-2 digit clue numbers displayed in top-left)
  - Letter display (user-entered letters shown in cell center)
  - Selection indicator (underscore when selected)
  - Yellow highlighting for selected cell borders
- Scrollbars (vertical and horizontal) when puzzle exceeds viewport
- Auto-scroll keeps selected cell visible during navigation

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
- **Game:**
  - Arrow keys: Navigate between cells (skips filled/black cells)
  - A-Z: Enter letter in current cell (auto-uppercased)
  - Backspace/Delete: Clear current cell
  - ECS: Quit to menu

## Development Notes

### Current State

- Puzzle loading and display working
- Menu system functional with Play/Exit options
- Grid rendering complete with proper border handling
- **Playable:** Users can navigate cells and enter letters
- Cell selection with yellow highlighting
- Auto-scroll keeps selection visible
- Scrollbars for large puzzles

### Known Limitations

- Puzzle date is hardcoded (2025-12-08) in app.rs:69
- Only one provider implemented (Lovatt's Cryptic)
- No rebus square support (noted in tui/src/lib.rs:15)
- No answer validation/checking yet
- No clue display panel yet
- Error handling incomplete in some areas (todo!() macros present)

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
