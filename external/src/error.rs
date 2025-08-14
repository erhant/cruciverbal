use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExternalError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Invalid file format: expected {expected}, got {actual}")]
    InvalidFormat { expected: String, actual: String },
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid word list format: {0}")]
    InvalidWordList(String),
}