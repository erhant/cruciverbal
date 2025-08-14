use crate::error::ExternalError;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordEntry {
    pub word: String,
    pub clue: String,
    pub difficulty: Option<u8>, // 1-5 difficulty rating
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordList {
    pub name: String,
    pub description: Option<String>,
    pub entries: Vec<WordEntry>,
    pub metadata: HashMap<String, String>,
}

impl WordList {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            entries: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, ExternalError> {
        let file = File::open(&path)?;
        let mut reader = csv::Reader::from_reader(file);
        let mut entries = Vec::new();

        for result in reader.deserialize() {
            let record: WordEntry = result?;
            entries.push(record);
        }

        let filename = path
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(Self {
            name: filename,
            description: Some("Imported from CSV".to_string()),
            entries,
            metadata: HashMap::new(),
        })
    }

    pub fn to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), ExternalError> {
        let file = File::create(path)?;
        let mut writer = csv::Writer::from_writer(file);

        for entry in &self.entries {
            writer.serialize(entry)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn from_json<P: AsRef<Path>>(path: P) -> Result<Self, ExternalError> {
        let file = File::open(path)?;
        let word_list: WordList = serde_json::from_reader(file)?;
        Ok(word_list)
    }

    pub fn to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), ExternalError> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    pub fn add_entry(&mut self, entry: WordEntry) {
        self.entries.push(entry);
    }

    pub fn get_words_by_length(&self, length: usize) -> Vec<&WordEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.word.len() == length)
            .collect()
    }

    pub fn get_words_by_category(&self, category: &str) -> Vec<&WordEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry
                    .category
                    .as_ref()
                    .map(|c| c == category)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn search_words(&self, pattern: &str) -> Vec<&WordEntry> {
        let pattern = pattern.to_lowercase();
        self.entries
            .iter()
            .filter(|entry| {
                entry.word.to_lowercase().contains(&pattern)
                    || entry.clue.to_lowercase().contains(&pattern)
            })
            .collect()
    }

    pub fn merge(&mut self, other: WordList) {
        self.entries.extend(other.entries);
        for (key, value) in other.metadata {
            self.metadata.insert(key, value);
        }
    }
}