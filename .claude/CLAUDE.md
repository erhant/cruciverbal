# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Run the application
cargo run -p cruciverbal

# Build release
cargo build --release

# Run all tests
cargo test

# Run provider tests only
cargo test -p cruciverbal-providers

# Run TUI tests only
cargo test -p cruciverbal-tui

# Run a specific test
cargo test -p cruciverbal-providers test_download_latest
```

## Project Overview

Cruciverbal is a terminal-based crossword puzzle player written in Rust (Edition 2024). It provides a TUI for solving crossword puzzles downloaded from 15 external providers.

**Tech Stack:**

- TUI Framework: Ratatui v0.29 with Crossterm
- Async Runtime: Tokio
- Puzzle Format: PUZ format via `puz-parse` crate
- HTTP Client: Reqwest

## Architecture

The project is a Cargo workspace with two crates:

### `tui/` - Terminal User Interface (binary: `cruciverbal`)

The main application with state machine architecture:

- `app.rs` - Core event loop at 35 FPS, view management, puzzle download orchestration
- `save.rs` - Save/load game functionality:
  - Explicit saves: `~/.cruciverbal/saves/`
  - Auto-saves (on ESC to menu): `~/.cruciverbal/autosaves/` - cleaned up after 7 days
- `views/menu.rs` - Main menu with navigation
- `views/help.rs` - Keyboard controls reference
- `views/game/mod.rs` - Game state, input handling, 3-area layout (top bar, grid, clue bar)
- `views/game/grid.rs` - `PuzzleGrid` struct with Unicode box-drawing rendering, cell/word selection
- `views/game/cell.rs` - `PuzzleCell`, `ClueNoDirection`, `WordIdxDirection` types

**Key State Types:**

- `AppView` enum: Menu, Help, Game(GameView)
- `GameView` enum: Selecting, LoadSelect, Loading, Playing, Completed, CompletedPlaying
- `GameState`: puzzle data, grid, selection position, timer, completion tracking

### `providers/` - Puzzle Providers Library (crate: `cruciverbal-providers`)

Library for fetching puzzles from external APIs:

- `lib.rs` - `PuzzleProvider` enum with 15 variants and `ALL` constant array
- `errors.rs` - `ProviderError` unified error type
- `util.rs` - HTTP client with user-agent, URL decoding
- `formats/crossword_compiler.rs` - CrosswordCompiler XML parser
- `providers/` - Individual provider implementations:
  - `guardian.rs` - 7 variants (Cryptic, Everyman, Speedy, Quick, Prize, Weekend, Quiptic) - latest only
  - `wapo.rs` - Washington Post Sunday - date-based and latest
  - `usa_today.rs` - USA Today - date-based and latest
  - `simply_daily.rs` - 3 variants (Regular, Cryptic, Quick) - date-based and latest
  - `universal.rs` - Universal - date-based and latest
  - `daily_pop.rs` - Daily Pop - date-based and latest
  - `lovatts_cryptic.rs` - Lovatts Cryptic - date-based only

**Provider Pattern:** Each provider module exposes `download(date: &str)` and optionally `download_latest()`, returning `Result<Puzzle, ProviderError>`.

## Key Implementation Details

**Grid Rendering:** Each cell is 4 lines tall × 4 chars wide using Unicode double-line box characters (╔╗╚╝╬). Yellow highlighting for cursor cell, cyan for word cells in active direction.

**Clue Numbering:** `PuzzleGrid::from_solution()` automatically assigns clue numbers and builds `ClueNoDirection`/`WordIdxDirection` for each cell.

**Completion Detection:** `PuzzleGrid::is_fully_correct()` compares user input against solution. Win triggers congratulations popup with time display.

## Testing Strategy

Tests are implemented as inline `#[cfg(test)]` modules within provider files. Each provider has integration tests that verify puzzle downloading:

- **Provider tests:** Located in each `providers/src/providers/*.rs` file
- **Format tests:** Located in `providers/src/formats/crossword_compiler.rs`
- **Grid tests:** Located in `tui/src/views/game/grid.rs`

Run tests with network access as they fetch from live APIs.

## Known Limitations

- No rebus square support (tui/src/lib.rs:9)
- Guardian providers only support "latest" mode (no date selection)
- `tui/src/components/mod.rs` is an empty placeholder for future UI components

## Configuration

- Clippy lint: `uninlined_format_args = "allow"` (tui/Cargo.toml)
- Workspace shares common dependency versions via `workspace.dependencies`
- Rust Edition 2024 (requires Rust 1.85+)
