mod binary_parser;
mod dispatcher;
mod error;
mod text_parser;

pub use dispatcher::PARSER_DISPATCHER;
pub use error::ParseError;

#[allow(unused_imports)] // May be used externally
pub use dispatcher::ParserDispatcher;

use mimetype_detector::kind::MimeKind;

/// Result of parsing a file
#[derive(Debug, Clone)]
pub struct ParsedContent {
    /// The extracted searchable text content
    pub text: String,
    /// The detected MIME type
    #[allow(dead_code)] // Will be used by future parsers
    pub mime_type: String,
    /// Optional metadata extracted from the file
    #[allow(dead_code)] // Will be used by future parsers
    pub metadata: Option<FileMetadata>,
}

/// Optional metadata that parsers can extract
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Will be used by future parsers (PDF, DOCX, etc.)
pub struct FileMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub created_date: Option<String>,
    pub page_count: Option<u32>,
    pub word_count: Option<u32>,
}

/// The core parser trait for extracting searchable text from files
pub trait Parser: Send + Sync {
    /// Human-readable name of the parser
    fn name(&self) -> &'static str;

    /// MIME kind categories this parser can handle
    /// Example: [MimeKind::Text, MimeKind::Application]
    fn supported_kinds(&self) -> &[MimeKind];

    /// File extensions this parser can handle (fallback for MIME detection)
    /// Example: ["rs", "py", "js"]
    fn supported_extensions(&self) -> &[&'static str];

    /// Parse raw bytes and extract searchable text
    ///
    /// # Arguments
    /// * `content` - Raw file bytes
    /// * `file_path` - Path to the file (for logging/context)
    ///
    /// # Returns
    /// * `Ok(ParsedContent)` - Successfully parsed content
    /// * `Err(ParseError)` - Failed to parse
    fn parse(&self, content: &[u8], file_path: &str) -> Result<ParsedContent, ParseError>;

    /// Priority for parser selection (higher = checked first)
    /// Default is 0; specialized parsers should return higher values
    fn priority(&self) -> i32 {
        0
    }
}
