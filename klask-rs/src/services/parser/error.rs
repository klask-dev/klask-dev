use thiserror::Error;

/// Errors that can occur during file parsing
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unsupported file type: {0}")]
    UnsupportedType(String),

    #[error("Binary file detected: {0}")]
    BinaryFile(String),

    #[error("Failed to parse file: {0}")]
    #[allow(dead_code)] // Will be used by future parsers (PDF, DOCX, etc.)
    ParseFailed(String),

    #[error("File too large: {size} bytes (max: {max_size})")]
    #[allow(dead_code)] // Will be used by future parsers
    FileTooLarge { size: u64, max_size: u64 },

    #[error("Invalid UTF-8 content")]
    InvalidUtf8,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
