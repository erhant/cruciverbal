# Cruciverbal

## Project Overview

Cruciverbal is a terminal-based crossword puzzle player written in Rust. The application provides a TUI (Terminal User Interface) for solving crossword puzzles from various providers. It supports 15 different puzzle providers including Guardian, Washington Post, USA Today, and more.

**Key Features:**

- Terminal-based crossword puzzle interface
- Async puzzle downloading from 15+ external providers
- Grid rendering with scrollable viewport
- Multiple puzzle format support (PUZ, CrosswordCompiler XML, JSON APIs)
- Interactive puzzle solving with keyboard navigation
- Auto-scrolling to keep selected cell visible
- Direction toggling (Across/Down) with word highlighting
- 3-area game layout: top bar (date, title, completion%, timer), grid, bottom bar (clue)
- Puzzle selection screen with date picker and provider selector
- "Latest puzzle" mode for providers that support it
- Completion tracking with percentage display and win detection
- Congratulations popup on puzzle completion with time display

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
  - `help.rs` - Help screen with keyboard controls
  - `game/` - Game view components
    - `mod.rs` - Game state, input handling, and navigation logic
    - `grid.rs` - PuzzleGrid struct with rendering and selection management
    - `cell.rs` - PuzzleCell, PuzzleCellValue, ClueNoDirection, WordIdxDirection
    - `constants.rs` - Unicode box-drawing character constants
- `components/` - Reusable UI components (currently empty, for future use)

**Key Components:**

- `App` - Main application state machine with view switching
- `AppView` enum - Menu, Help, or Game view states
- `MenuItem` enum - Menu items (NewGame, Help, Exit)
- `GameView` enum - Playing, Selecting, Loading, Saving, Completed, CompletedPlaying states
- `CompletionState` enum - InProgress, IncorrectFill, Correct
- `GameState` - Game state including:
  - `puzzle: Option<Puzzle>` - Raw puzzle data from puz_parse
  - `grid: Option<PuzzleGrid>` - Playable grid built from puzzle
  - `sel: (usize, usize)` - Currently selected cell (row, col)
  - `active_direction: Direction` - Current direction for clue display and word highlighting
  - `puzzle_date: Option<String>` - Date of the loaded puzzle
  - `start_time: Option<Instant>` - Timer start time
  - `selection: SelectionState` - State for puzzle selection screen
  - `completion_state: CompletionState` - Current completion state
  - `completion_time: Option<Duration>` - Final time when puzzle is completed
  - `completed_popup_selection: usize` - Selected option in completion popup
  - `visible_area`, `scroll_cur`, `scroll_max`, `scroll_bar` - Viewport/scrolling state
- `SelectionState` - State for puzzle selection:
  - `date: String` - Date input (YYYY-MM-DD)
  - `use_latest: bool` - Whether to use "latest" mode instead of specific date
  - `provider_idx: usize` - Selected provider index (0-14)
  - `active_field: SelectionField` - Which field is active (Date/Provider/Start)
  - `error: Option<String>` - Error message to display
- `PuzzleProvider` enum - 15 available providers (from cruciverbal_providers crate)
- `Direction` enum - Across or Down direction
- `MenuState` - Menu selection tracking
- `PuzzleGrid` - Grid with cells, selection management, and rendering:
  - `from_solution(&[String])` - Build grid from puz_parse solution with automatic clue numbering
  - `set_selection(row, col, direction)` - Cell and word selection with highlighting
  - `get()`, `get_mut()` - Cell access
  - `to_par()` - Convert to Ratatui Paragraph for rendering
  - `reveal_word(clue_no, direction)` - Reveal all letters in a word
  - `reveal_all()` - Reveal entire puzzle
  - `count_total_letters()` - Count all letter cells
  - `count_filled_letters()` - Count cells with user input
  - `completion_percentage()` - Get fill percentage (0-100)
  - `is_fully_correct()` - Check if all letters match solution
- `PuzzleCell` - Individual cell with:
  - `val: PuzzleCellValue` - Either `Filled` (black) or `Letter` with clue data
  - `is_selected_cell: bool` - Whether this is the cursor cell (yellow)
  - `is_selected_word: bool` - Whether this cell is part of the selected word (cyan)
  - `set_user_letter()`, `get_user_letter()` - User input management
  - `reveal()` - Set user_letter to clue_letter (reveal correct answer)
  - `is_empty()` - Check if cell has no user input
  - `is_correct()` - Check if user_letter matches clue_letter
  - `clue_no_for_direction()`, `word_idx_for_direction()`, `has_direction()` - Direction helpers
- `ClueNoDirection` - Tracks which clue number(s) the cell belongs to (Across/Down/Cross)
- `WordIdxDirection` - Tracks 0-based position within the word(s)

### 2. `providers/` - Puzzle Providers Library

Library crate for fetching puzzles from external sources.

**Structure:**

- `lib.rs` - Module exports, `PuzzleProvider` enum with all 15 providers
- `errors.rs` - Provider error types
- `util.rs` - Shared utilities (HTTP client, URL decoding, user-agent string)
- `formats/` - Puzzle format parsers
  - `crossword_compiler.rs` - CrosswordCompiler XML format parser
- `providers/` - Individual provider implementations
  - `lovatts_cryptic.rs` - Lovatt's Cryptic crossword provider
  - `guardian.rs` - Guardian crosswords (7 variants: Cryptic, Everyman, Speedy, Quick, Prize, Weekend, Quiptic)
  - `wapo.rs` - Washington Post Sunday crossword
  - `usa_today.rs` - USA Today crossword
  - `simply_daily.rs` - Simply Daily Puzzles (3 variants: Regular, Cryptic, Quick)
  - `universal.rs` - Universal crossword (via AMUniversal API)
  - `daily_pop.rs` - Daily Pop crossword (via PuzzleNation API)

**Key Components:**

- `PuzzleProvider` enum - All 15 available providers with `ALL` constant array
- `ProviderError` - Unified error type for fetch/parse failures
- `GuardianVariant` enum - 7 Guardian crossword types
- `SimplyDailyVariant` enum - 3 Simply Daily puzzle types
- Provider modules implement `download(date)` and often `download_latest()` functions
- `util::http_client()` - Configured reqwest client with user-agent
- `util::url_decode()` - URL percent-encoding decoder
- `formats::crossword_compiler::parse()` - XML parser for CrosswordCompiler format

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
- Date input field (YYYY-MM-DD, defaults to today) with "latest" mode toggle
- Provider selector (15 providers available, use ←→ to cycle)
- Start button
- Error message display for validation/download failures

### Puzzle Providers

The application supports 15 puzzle providers organized into families:

**Lovatts (providers/src/providers/lovatts_cryptic.rs)**
- Fetches from Puzzleexperts API with custom data format
- Date-based download only

**Guardian (providers/src/providers/guardian.rs)**
- 7 variants: Cryptic, Everyman, Speedy, Quick, Prize, Weekend, Quiptic
- Scrapes HTML to extract JSON from `<gu-island>` component
- Latest puzzle download only (no date selection)

**Washington Post (providers/src/providers/wapo.rs)**
- Sunday crosswords via REST API
- Date-based (YYYY/MM/DD format) and latest (most recent Sunday) download

**USA Today (providers/src/providers/usa_today.rs)**
- XML format via uclick.com API
- Date-based and latest download with retry fallback

**Simply Daily (providers/src/providers/simply_daily.rs)**
- 3 variants: Regular, Cryptic, Quick
- Uses CrosswordCompiler XML format (JS-embedded)
- Date-based and latest download

**Universal (providers/src/providers/universal.rs)**
- Via AMUniversal JSON API
- Date-based and latest download with retry logic

**Daily Pop (providers/src/providers/daily_pop.rs)**
- Via PuzzleNation API with dynamic API key fetching
- Uses CrosswordCompiler XML format
- Date-based and latest download

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
  - SHIFT + Arrow keys: Jump to next empty cell in a different clue word
  - A-Z: Enter letter in current cell (auto-uppercased)
  - Backspace/Delete: Clear current cell
  - SPACEBAR: Toggle direction (Across ↔ Down)
  - CTRL+R: Reveal current letter
  - SHIFT+CTRL+R: Reveal current word
  - ALT+CTRL+R: Reveal entire puzzle
  - ESC: Back to menu

## Development Notes

### Current State

- **Fully playable** crossword puzzle experience
- **15 puzzle providers** implemented with date and/or "latest" modes
- Menu system with New Game and Help options
- Help screen with keyboard controls reference
- Puzzle selection screen with date picker, "latest" toggle, and provider selector
- Async puzzle download with error handling and retry logic
- Multiple puzzle format support (custom JSON, XML, CrosswordCompiler)
- Grid rendering with proper border handling
- Cell and word selection with direction toggling
- Yellow highlighting for cursor, cyan for word
- Clue display in bottom bar based on active direction
- Timer display in top bar
- Completion percentage tracking in top bar
- Win detection with congratulations popup
- Post-completion browsing mode (timer frozen)
- Auto-scroll keeps selection visible
- Scrollbars for large puzzles
- Reveal functionality (letter, word, or full puzzle)

### Known Limitations

- No rebus square support (noted in tui/src/lib.rs:12)
- No save/load game progress
- Error handling incomplete in some areas (todo!() macros present)
- GameView::Saving not implemented
- Guardian providers only support "latest" mode (no date selection)
- Some providers have limited puzzle archive availability (WaPo only keeps recent puzzles)

### Planned Features (from code TODOs)

From tui/src/lib.rs:

- Save progress locally (CTRL+S)
- Rebus square support (multiple letters per cell)

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

**Providers-specific:**

- `quick-xml` v0.37 with serialize feature for XML parsing (CrosswordCompiler format)
- `reqwest` v0.11 for HTTP requests
- `chrono` v0.4 for date handling

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

- `providers/src/providers/lovatts_cryptic.rs` - Download test (async)
- `providers/src/providers/guardian.rs` - Download tests for Cryptic and Quick variants
- `providers/src/providers/wapo.rs` - Download tests (date-based and latest)
- `providers/src/providers/usa_today.rs` - Download tests (date-based and latest)
- `providers/src/providers/simply_daily.rs` - Download tests for all 3 variants
- `providers/src/providers/universal.rs` - Download tests (date-based and latest)
- `providers/src/providers/daily_pop.rs` - Download tests (date-based and latest)
- `providers/src/formats/crossword_compiler.rs` - XML extraction tests
- `providers/src/util.rs` - URL decode tests
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
