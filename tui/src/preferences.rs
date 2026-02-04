//! User preferences persistence.
//!
//! Stores user preferences in `~/.cruciverbal/preferences.json`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Error type for preferences operations.
#[derive(Error, Debug)]
pub enum PreferencesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Could not determine home directory")]
    NoHomeDir,
}

/// User preferences.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Preferences {
    /// The selected theme ID.
    #[serde(default = "default_theme_id")]
    pub theme_id: String,
}

fn default_theme_id() -> String {
    "default".to_string()
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme_id: default_theme_id(),
        }
    }
}

/// Get the preferences file path (`~/.cruciverbal/preferences.json`).
pub fn preferences_path() -> Result<PathBuf, PreferencesError> {
    let home = dirs::home_dir().ok_or(PreferencesError::NoHomeDir)?;
    Ok(home.join(".cruciverbal").join("preferences.json"))
}

/// Load preferences from disk.
///
/// Returns default preferences if the file doesn't exist or can't be read.
pub fn load_preferences() -> Preferences {
    let path = match preferences_path() {
        Ok(p) => p,
        Err(_) => return Preferences::default(),
    };

    if !path.exists() {
        return Preferences::default();
    }

    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Preferences::default(),
    };

    serde_json::from_str(&contents).unwrap_or_default()
}

/// Save preferences to disk.
pub fn save_preferences(prefs: &Preferences) -> Result<(), PreferencesError> {
    let path = preferences_path()?;

    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(prefs)?;
    std::fs::write(&path, json)?;

    Ok(())
}
