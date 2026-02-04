//! Save/Load game functionality.
//!
//! Explicit saves go to `~/.cruciverbal/saves/`, auto-saves go to `~/.cruciverbal/autosaves/`.

use crate::views::game::{CompletionState, Direction};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Error type for save/load operations.
#[derive(Error, Debug)]
pub enum SaveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Could not determine home directory")]
    NoHomeDir,
    #[error("Invalid save file: {0}")]
    InvalidSave(String),
}

/// Serializable game save data.
#[derive(Serialize, Deserialize)]
pub struct GameSave {
    /// Version for future compatibility.
    pub version: u8,
    /// Puzzle date string (e.g., "2025-01-30" or "Latest").
    pub puzzle_date: String,
    /// Provider name for display.
    pub provider_name: String,
    /// Provider index in PuzzleProvider::ALL.
    pub provider_idx: usize,
    /// The full puzzle data (uses puz-parse's serde feature).
    pub puzzle: puz_parse::Puzzle,
    /// User-entered letters grid (None = empty cell).
    pub user_letters: Vec<Vec<Option<char>>>,
    /// Elapsed time in seconds at save.
    pub elapsed_secs: u64,
    /// Selected cell position (row, col).
    pub sel: (usize, usize),
    /// Active direction (Across or Down).
    pub active_direction: Direction,
    /// Completion state at save time.
    pub completion_state: CompletionState,
    /// Whether this was an auto-save (ESC to menu) vs explicit save (CTRL+S).
    #[serde(default)]
    pub is_auto_save: bool,
    /// Timestamp when saved (Unix epoch seconds).
    #[serde(default)]
    pub saved_at: u64,
}

/// Get the saves directory path (`~/.cruciverbal/saves/`).
pub fn saves_dir() -> Result<PathBuf, SaveError> {
    let home = dirs::home_dir().ok_or(SaveError::NoHomeDir)?;
    Ok(home.join(".cruciverbal").join("saves"))
}

/// Get the auto-saves directory path (`~/.cruciverbal/autosaves/`).
pub fn autosaves_dir() -> Result<PathBuf, SaveError> {
    let home = dirs::home_dir().ok_or(SaveError::NoHomeDir)?;
    Ok(home.join(".cruciverbal").join("autosaves"))
}

/// Generate a filename for a save: `{date}_{provider-slug}.json`.
fn generate_filename(date: &str, provider_name: &str) -> String {
    let slug = provider_name
        .to_lowercase()
        .replace(' ', "-")
        .replace('/', "-");
    format!("{}_{}.json", date, slug)
}

/// Save a game to disk.
///
/// Returns the path where the save was written.
/// Explicit saves go to `~/.cruciverbal/saves/`, auto-saves go to `~/.cruciverbal/autosaves/`.
pub fn save_game(save: &GameSave) -> Result<PathBuf, SaveError> {
    let dir = if save.is_auto_save {
        autosaves_dir()?
    } else {
        saves_dir()?
    };
    std::fs::create_dir_all(&dir)?;

    let filename = generate_filename(&save.puzzle_date, &save.provider_name);
    let path = dir.join(filename);

    let json = serde_json::to_string_pretty(save)?;
    std::fs::write(&path, json)?;

    Ok(path)
}

/// Load a game from a save file.
pub fn load_game(path: &Path) -> Result<GameSave, SaveError> {
    let contents = std::fs::read_to_string(path)?;
    let save: GameSave = serde_json::from_str(&contents)?;

    // Version check for future compatibility
    if save.version != 1 {
        return Err(SaveError::InvalidSave(format!(
            "Unsupported save version: {}",
            save.version
        )));
    }

    Ok(save)
}

/// Information about a saved game for display in the load menu.
#[derive(Debug, Clone)]
pub struct SaveInfo {
    /// Full path to the save file.
    pub path: PathBuf,
    /// Puzzle date string.
    pub date: String,
    /// Provider name.
    pub provider: String,
    /// Completion percentage (0-100).
    pub completion_pct: u8,
}

/// List all saved games (explicit saves only).
///
/// Returns a vector of save info sorted by modification time (newest first).
pub fn list_saves() -> Result<Vec<SaveInfo>, SaveError> {
    list_saves_in_dir(&saves_dir()?)
}

/// List all auto-saves (for Recently Played).
///
/// Returns a vector of save info sorted by modification time (newest first).
pub fn list_autosaves() -> Result<Vec<SaveInfo>, SaveError> {
    list_saves_in_dir(&autosaves_dir()?)
}

/// List all saves in a directory.
fn list_saves_in_dir(dir: &Path) -> Result<Vec<SaveInfo>, SaveError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut saves: Vec<(SaveInfo, std::time::SystemTime)> = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "json") {
            // Try to load the save to get metadata
            if let Ok(save) = load_game(&path) {
                let completion_pct = calculate_completion_pct(&save.user_letters);
                let mtime = entry.metadata()?.modified()?;

                saves.push((
                    SaveInfo {
                        path,
                        date: save.puzzle_date,
                        provider: save.provider_name,
                        completion_pct,
                    },
                    mtime,
                ));
            }
        }
    }

    // Sort by modification time (newest first)
    saves.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(saves.into_iter().map(|(info, _)| info).collect())
}

/// Calculate completion percentage from user letters grid.
fn calculate_completion_pct(user_letters: &[Vec<Option<char>>]) -> u8 {
    let total: usize = user_letters.iter().flatten().count();
    if total == 0 {
        return 100;
    }
    let filled: usize = user_letters
        .iter()
        .flatten()
        .filter(|c| c.is_some())
        .count();
    ((filled * 100) / total) as u8
}

/// Delete a save file.
pub fn delete_save(path: &Path) -> Result<(), SaveError> {
    std::fs::remove_file(path)?;
    Ok(())
}

/// Delete auto-saves older than 7 days.
///
/// Returns the number of deleted saves.
pub fn cleanup_old_auto_saves() -> Result<usize, SaveError> {
    let dir = autosaves_dir()?;
    if !dir.exists() {
        return Ok(0);
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let seven_days_secs: u64 = 7 * 24 * 60 * 60;
    let cutoff = now.saturating_sub(seven_days_secs);

    let mut deleted = 0;

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "json") {
            if let Ok(save) = load_game(&path) {
                // Delete auto-saves older than 7 days
                if save.saved_at > 0 && save.saved_at < cutoff {
                    if delete_save(&path).is_ok() {
                        deleted += 1;
                    }
                }
            }
        }
    }

    Ok(deleted)
}
