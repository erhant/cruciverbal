use crate::error::ExternalError;
use cruciverbal_core::Crossword;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    path::Path,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameFileFormat {
    Json,
    Binary,
}

impl GameFileFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(GameFileFormat::Json),
            "cwb" => Some(GameFileFormat::Binary), // Crossword Binary
            _ => None,
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            GameFileFormat::Json => "json",
            GameFileFormat::Binary => "cwb",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameFileMetadata {
    pub version: String,
    pub created_at: String,
    pub modified_at: String,
    pub author: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameFile {
    pub metadata: GameFileMetadata,
    pub crossword: Crossword,
}

impl GameFile {
    pub fn new(crossword: Crossword) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        
        Self {
            metadata: GameFileMetadata {
                version: "1.0".to_string(),
                created_at: now.clone(),
                modified_at: now,
                author: crossword.author.clone(),
                description: None,
            },
            crossword,
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ExternalError> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ExternalError::InvalidFormat {
                expected: "json or cwb".to_string(),
                actual: "unknown".to_string(),
            })?;

        let format = GameFileFormat::from_extension(extension)
            .ok_or_else(|| ExternalError::InvalidFormat {
                expected: "json or cwb".to_string(),
                actual: extension.to_string(),
            })?;

        match format {
            GameFileFormat::Json => Self::load_json(path),
            GameFileFormat::Binary => Self::load_binary(path),
        }
    }

    pub fn save<P: AsRef<Path>>(&mut self, path: P, format: GameFileFormat) -> Result<(), ExternalError> {
        // Update modification time
        self.metadata.modified_at = chrono::Utc::now().to_rfc3339();

        match format {
            GameFileFormat::Json => self.save_json(path),
            GameFileFormat::Binary => self.save_binary(path),
        }
    }

    fn load_json<P: AsRef<Path>>(path: P) -> Result<Self, ExternalError> {
        let file = File::open(path)?;
        let game_file: GameFile = serde_json::from_reader(file)?;
        Ok(game_file)
    }

    fn save_json<P: AsRef<Path>>(&self, path: P) -> Result<(), ExternalError> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    fn load_binary<P: AsRef<Path>>(path: P) -> Result<Self, ExternalError> {
        let file = File::open(path)?;
        let game_file: GameFile = bincode::deserialize_from(file)
            .map_err(|_| ExternalError::InvalidFormat {
                expected: "valid binary crossword file".to_string(),
                actual: "corrupted or invalid binary data".to_string(),
            })?;
        Ok(game_file)
    }

    fn save_binary<P: AsRef<Path>>(&self, path: P) -> Result<(), ExternalError> {
        let file = File::create(path)?;
        bincode::serialize_into(file, self)
            .map_err(|_| ExternalError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to serialize crossword to binary format",
            )))?;
        Ok(())
    }

    pub fn export_to_text<P: AsRef<Path>>(&self, path: P) -> Result<(), ExternalError> {
        let content = self.to_text_format();
        std::fs::write(path, content)?;
        Ok(())
    }

    fn to_text_format(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("Title: {}\n", self.crossword.title));
        output.push_str(&format!("Author: {}\n", self.crossword.author));
        output.push_str(&format!("Created: {}\n", self.metadata.created_at));
        output.push_str("\n");

        // Grid representation
        output.push_str("Grid:\n");
        for row in &self.crossword.grid.cells {
            for cell in row {
                output.push(cell.get_display_char());
            }
            output.push('\n');
        }
        output.push_str("\n");

        // Clues
        output.push_str("Across:\n");
        for clue in self.crossword.get_clues_by_direction(cruciverbal_core::Direction::Across) {
            output.push_str(&format!("{}. {} ({})\n", clue.number, clue.text, clue.answer));
        }
        
        output.push_str("\nDown:\n");
        for clue in self.crossword.get_clues_by_direction(cruciverbal_core::Direction::Down) {
            output.push_str(&format!("{}. {} ({})\n", clue.number, clue.text, clue.answer));
        }

        output
    }
}