# Cruciverbal

A terminal-based crossword puzzle player written in Rust. Solve crosswords from 15+ providers directly in your terminal.

## Features

- **15 puzzle providers** including Guardian, Washington Post, USA Today, and more
- **Interactive TUI** with keyboard navigation and word highlighting
- **Timer and completion tracking** with win detection
- **Reveal functionality** for hints (letter, word, or full puzzle)
- **Auto-scrolling** viewport for large puzzles

### Supported Providers

| Provider        | Variants                                                  |
| --------------- | --------------------------------------------------------- |
| Lovatts         | Cryptic                                                   |
| The Guardian    | Cryptic, Everyman, Speedy, Quick, Prize, Weekend, Quiptic |
| Washington Post | Sunday                                                    |
| USA Today       | Daily                                                     |
| Simply Daily    | Regular, Cryptic, Quick                                   |
| Universal       | Daily                                                     |
| Daily Pop       | Daily                                                     |

## Installation

### Via Cargo

```bash
cargo install cruciverbal
```

### From Source

```bash
git clone https://github.com/erhant/cruciverbal.git
cd cruciverbal
cargo build --release
```

The binary will be at `target/release/cruciverbal`.

## Usage

Run the application:

```bash
cruciverbal
```

Or if built from source:

```bash
cargo run -p cruciverbal
```

### Keyboard Controls

#### Navigation

| Key           | Action                         |
| ------------- | ------------------------------ |
| Arrow keys    | Move between cells             |
| Shift + Arrow | Jump to next word              |
| Space         | Toggle direction (Across/Down) |

#### Input

| Key              | Action       |
| ---------------- | ------------ |
| A-Z              | Enter letter |
| Backspace/Delete | Clear cell   |

#### Reveal

| Key          | Action                |
| ------------ | --------------------- |
| Ctrl+R       | Reveal current letter |
| Shift+Ctrl+R | Reveal current word   |
| Alt+Ctrl+R   | Reveal entire puzzle  |

#### General

| Key    | Action           |
| ------ | ---------------- |
| ESC    | Back to menu     |
| Ctrl+C | Quit application |

## License

[MIT](./LICENSE)

## References

- [thisisparker/xword-dl](https://github.com/thisisparker/xword-dl)
- [thisisparker/cursewords](https://github.com/thisisparker/cursewords)
- [apexatoll/cliptic](https://github.com/apexatoll/cliptic)
- [Daily Crossword Links](https://dailycrosswordlinks.com/)
- [Crossword Fiend](https://crosswordfiend.com/download/)
